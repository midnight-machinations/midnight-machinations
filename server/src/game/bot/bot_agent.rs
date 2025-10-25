use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        ChatCompletionTool, ChatCompletionToolType, FunctionObjectArgs,
        ChatCompletionToolChoiceOption,
    },
};
use serde_json::json;

use crate::{
    game::controllers::{Controller, ControllerID, ControllerInput},
    packet::ToClientPacket,
    room::RoomClientID,
    vec_map::VecMap,
};

/// Bot agent that uses an LLM to make decisions in the game
pub struct BotAgent {
    #[allow(dead_code, reason = "Used for logging and debugging")]
    player_id: RoomClientID,
    player_name: String,
    receiver: mpsc::UnboundedReceiver<ToClientPacket>,
    controller_sender: mpsc::UnboundedSender<ControllerInput>,
    openai_client: Client<async_openai::config::OpenAIConfig>,
    game_state: Arc<Mutex<BotGameState>>,
}

#[derive(Default)]
struct BotGameState {
    role: Option<String>,
    phase: Option<String>,
    alive_status: Vec<bool>,
    recent_messages: Vec<String>,
    available_controllers: VecMap<ControllerID, Controller>,
    player_index: Option<u8>,
}

impl BotAgent {
    pub fn new(
        player_id: RoomClientID,
        player_name: String,
        receiver: mpsc::UnboundedReceiver<ToClientPacket>,
        controller_sender: mpsc::UnboundedSender<ControllerInput>,
        api_key: Option<String>,
    ) -> Self {
        let config = if let Some(key) = api_key {
            async_openai::config::OpenAIConfig::new().with_api_key(key)
        } else {
            async_openai::config::OpenAIConfig::default()
        };

        Self {
            player_id,
            player_name,
            receiver,
            controller_sender,
            openai_client: Client::with_config(config),
            game_state: Arc::new(Mutex::new(BotGameState::default())),
        }
    }

    /// Start the bot agent in a separate thread
    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(packet) = self.receiver.recv().await {
                if let Err(e) = self.process_packet(packet).await {
                    eprintln!("Bot {} error processing packet: {}", self.player_name, e);
                }
            }
        })
    }

    async fn process_packet(&mut self, packet: ToClientPacket) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.game_state.lock().await;

        match packet {
            ToClientPacket::YourPlayerIndex { player_index } => {
                state.player_index = Some(player_index);
            }
            ToClientPacket::YourRole { role } => {
                state.role = Some(format!("{:?}", role));
            }
            ToClientPacket::Phase { phase, .. } => {
                state.phase = Some(format!("{:?}", phase));
                // When phase changes, consider making a decision
                drop(state);
                self.make_decision().await?;
            }
            ToClientPacket::PlayerAlive { alive } => {
                state.alive_status = alive;
            }
            ToClientPacket::AddChatMessages { chat_messages } => {
                for (_, msg) in chat_messages.iter() {
                    let msg_str = format!("{:?}", msg);
                    state.recent_messages.push(msg_str);
                    if state.recent_messages.len() > 20 {
                        state.recent_messages.remove(0);
                    }
                }
            }
            ToClientPacket::YourAllowedControllers { save } => {
                state.available_controllers = save;
                // When we get new controllers, we might want to use them
                drop(state);
                self.make_decision().await?;
            }
            ToClientPacket::YourAllowedController { id, controller } => {
                if let Some(controller) = controller {
                    state.available_controllers.insert(id, controller);
                } else {
                    state.available_controllers.remove(&id);
                }
                drop(state);
                self.make_decision().await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn make_decision(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.game_state.lock().await;
        
        // Don't make decisions if we don't have basic info
        if state.player_index.is_none() || state.role.is_none() {
            return Ok(());
        }

        if state.available_controllers.is_empty() {
            return Ok(());
        }
        
        // Build context for the LLM
        let context = format!(
            "You are playing a social deduction game called Midnight Machinations.\n\
             Your name: {}\n\
             Your role: {}\n\
             Current phase: {}\n\
             Alive players: {:?}\n\
             Recent messages: {}\n\n\
             You have controllers available to send actions. Consider your role's objectives and make strategic decisions.\n\
             Use the send_ability_input tool to perform game actions, or send_chat_message to communicate with other players.",
            self.player_name,
            state.role.as_ref().unwrap_or(&"Unknown".to_string()),
            state.phase.as_ref().unwrap_or(&"Unknown".to_string()),
            state.alive_status,
            state.recent_messages.last().unwrap_or(&"None".to_string())
        );

        // Serialize available controllers to show the bot what's available
        let controllers_json = serde_json::to_string_pretty(&state.available_controllers)?;

        drop(state);

        // Define tools for the bot to use
        let tools = vec![
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObjectArgs::default()
                    .name("send_chat_message")
                    .description("Send a chat message to other players")
                    .parameters(json!({
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "The message to send"
                            }
                        },
                        "required": ["message"]
                    }))
                    .build()?,
            },
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObjectArgs::default()
                    .name("send_ability_input")
                    .description(format!(
                        "Send a controller input to perform a game action. The input must be a ControllerInput object with 'id' and 'selection' fields.\n\n\
                        ControllerID format:\n\
                        - For role abilities: {{\"type\": \"role\", \"player\": <player_index>, \"role\": \"<RoleName>\", \"id\": <ability_id>}}\n\
                        - For chat: {{\"type\": \"sendChat\", \"player\": <player_index>}}\n\
                        - For nomination: {{\"type\": \"nominate\", \"player\": <player_index>}}\n\
                        - For judgement: {{\"type\": \"judge\", \"player\": <player_index>}}\n\n\
                        ControllerSelection format (choose based on controller type):\n\
                        - Unit: {{\"type\": \"unit\", \"selection\": null}}\n\
                        - Boolean: {{\"type\": \"boolean\", \"selection\": true/false}}\n\
                        - PlayerList: {{\"type\": \"playerList\", \"selection\": [player_indices]}}\n\
                        - String: {{\"type\": \"string\", \"selection\": \"text\"}}\n\
                        - Integer: {{\"type\": \"integer\", \"selection\": number}}\n\n\
                        Available controllers:\n{}", 
                        controllers_json
                    ))
                    .parameters(json!({
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "object",
                                "description": "The controller ID"
                            },
                            "selection": {
                                "type": "object",
                                "description": "The controller selection"
                            }
                        },
                        "required": ["id", "selection"]
                    }))
                    .build()?,
            },
        ];

        // Query the LLM with tools
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are an AI playing a social deduction game. Make strategic decisions using the provided tools. You can send chat messages or ability inputs based on your role and the game situation.")
                    .build()?
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(context)
                    .build()?
            ),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(messages)
            .tools(tools)
            .tool_choice(ChatCompletionToolChoiceOption::Auto)
            .max_tokens(200_u32)
            .temperature(0.7)
            .build()?;

        // Try to get a response from the LLM
        match self.openai_client.chat().create(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    // Check if the bot made tool calls
                    if let Some(tool_calls) = &choice.message.tool_calls {
                        for tool_call in tool_calls {
                            let function_name = &tool_call.function.name;
                            let arguments = &tool_call.function.arguments;
                            
                            println!("Bot {} calling tool: {} with args: {}", self.player_name, function_name, arguments);
                            
                            match function_name.as_str() {
                                "send_chat_message" => {
                                    if let Ok(args) = serde_json::from_str::<serde_json::Value>(arguments) {
                                        if let Some(message) = args.get("message").and_then(|m| m.as_str()) {
                                            // Find the chat controller
                                            let state = self.game_state.lock().await;
                                            for (id, _) in state.available_controllers.iter() {
                                                if matches!(id, ControllerID::SendChat { .. }) {
                                                    let input = ControllerInput::new(
                                                        id.clone(),
                                                        crate::game::controllers::StringSelection(message.to_string())
                                                    );
                                                    let _ = self.controller_sender.send(input);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                "send_ability_input" => {
                                    // Deserialize the controller input directly from the arguments
                                    match serde_json::from_str::<ControllerInput>(arguments) {
                                        Ok(controller_input) => {
                                            println!("Bot {} sending controller input: {:?}", self.player_name, controller_input);
                                            let _ = self.controller_sender.send(controller_input);
                                        }
                                        Err(e) => {
                                            eprintln!("Bot {} failed to deserialize controller input: {}. Args: {}", 
                                                self.player_name, e, arguments);
                                        }
                                    }
                                }
                                _ => {
                                    eprintln!("Bot {} called unknown tool: {}", self.player_name, function_name);
                                }
                            }
                        }
                    } else if let Some(content) = &choice.message.content {
                        println!("Bot {} thinking: {}", self.player_name, content);
                    }
                }
            }
            Err(e) => {
                eprintln!("Bot {} LLM error: {}", self.player_name, e);
            }
        }

        Ok(())
    }
}
