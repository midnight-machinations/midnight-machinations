use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestToolMessageArgs, CreateChatCompletionRequestArgs,
        ChatCompletionTool, ChatCompletionToolType, FunctionObjectArgs,
        ChatCompletionToolChoiceOption,
    },
};
use serde_json::json;

use crate::{
    game::{controllers::{Controller, ControllerID, ControllerInput}, player::PlayerReference}, log, packet::ToClientPacket, room::RoomClientID, vec_map::VecMap
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
    conversation_history: Vec<ChatCompletionRequestMessage>,
    last_decision_time: Option<Instant>,
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

        // Initialize conversation with system message
        let system_message = ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are an AI playing a social deduction game called Midnight Machinations. Make strategic decisions using the provided tools. You can send chat messages or ability inputs based on your role and the game situation. If you have nothing to do or say at the moment, use the no_op tool to indicate you're waiting.")
                .build()
                .unwrap()
        );

        Self {
            player_id,
            player_name,
            receiver,
            controller_sender,
            openai_client: Client::with_config(config),
            game_state: Arc::new(Mutex::new(BotGameState::default())),
            conversation_history: vec![system_message],
            last_decision_time: None,
        }
    }

    /// Start the bot agent in a separate thread
    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut decision_interval = tokio::time::interval(Duration::from_secs(7));
            decision_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                tokio::select! {
                    Some(packet) = self.receiver.recv() => {
                        if let Err(e) = self.process_packet(packet).await {
                            eprintln!("Bot {} error processing packet: {}", self.player_name, e);
                        }
                    }
                    _ = decision_interval.tick() => {
                        // Time-based decision making
                        if let Err(e) = self.make_decision().await {
                            eprintln!("Bot {} error making decision: {}", self.player_name, e);
                        }
                    }
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
                // Phase changes are important, so we update our context but don't immediately call LLM
                // The timer will handle it
            }
            ToClientPacket::PlayerAlive { alive } => {
                state.alive_status = alive;
            }
            ToClientPacket::AddChatMessages { chat_messages } => {
                for (_, msg) in chat_messages.iter() {
                    let msg_str = format!("{:?}", msg);
                    state.recent_messages.push(msg_str);
                    if state.recent_messages.len() > 100 {
                        state.recent_messages.remove(0);
                    }
                }
            }
            ToClientPacket::YourAllowedControllers { save } => {
                state.available_controllers = save;
                // Controllers updated, but we'll wait for the timer to make a decision
            }
            ToClientPacket::YourAllowedController { id, controller } => {
                if let Some(controller) = controller {
                    state.available_controllers.insert(id, controller);
                } else {
                    state.available_controllers.remove(&id);
                }
                // Controllers updated, but we'll wait for the timer to make a decision
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
            "Current game state:\n\
             Your name: {}\n\
             Your role: {}\n\
             Current phase: {}\n\
             Alive: {:?}\n\
             Recent messages: {:?}\n\n\
             You have controllers available to send actions. Consider your role's objectives and make strategic decisions.\n\
             Use the send_ability_input tool to perform game actions, send_chat_message to communicate with other players, or no_op if you have nothing to do right now.",
            self.player_name,
            state.role.as_ref().unwrap_or(&"Unknown".to_string()),
            state.phase.as_ref().unwrap_or(&"Unknown".to_string()),
            state.alive_status,
            state.recent_messages.iter().rev().take(5).collect::<Vec<_>>()
        );

        // Serialize available controllers to show the bot what's available
        let controllers_json = serde_json::to_string_pretty(&state.available_controllers)?;

        drop(state);

        // Define tools for the bot to use
        let tools = vec![
            ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObjectArgs::default()
                    .name("no_op")
                    .description("Do nothing. Use this when you have nothing to say or do at the moment, or when you're waiting for other players to act.")
                    .parameters(json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }))
                    .build()?,
            },
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

        // Add the new user message to conversation history
        let user_message = ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(context)
                .build()?
        );
        self.conversation_history.push(user_message);

        // Keep conversation history manageable (system message + last 20 messages)
        if self.conversation_history.len() > 21 {
            // Keep system message (index 0) and remove oldest messages
            self.conversation_history.drain(1..self.conversation_history.len() - 20);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(self.conversation_history.clone())
            .tools(tools)
            .tool_choice(ChatCompletionToolChoiceOption::Auto)
            .max_tokens(200_u32)
            .temperature(0.7)
            .build()?;

        // Try to get a response from the LLM
        match self.openai_client.chat().create(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    // Add assistant's response to conversation history
                    let mut assistant_builder = ChatCompletionRequestAssistantMessageArgs::default();
                    assistant_builder.content(choice.message.content.clone().unwrap_or_default());
                    
                    if let Some(tool_calls) = &choice.message.tool_calls {
                        assistant_builder.tool_calls(tool_calls.clone());
                    }
                    
                    if let Ok(assistant_message) = assistant_builder.build() {
                        self.conversation_history.push(ChatCompletionRequestMessage::Assistant(assistant_message));
                    }

                    // Check if the bot made tool calls
                    if let Some(tool_calls) = &choice.message.tool_calls {
                        for tool_call in tool_calls {
                            let function_name = &tool_call.function.name;
                            let arguments = &tool_call.function.arguments;
                            let tool_call_id = &tool_call.id;
                            
                            log!(info "Bot"; "Bot {} calling tool: {}", self.player_name, function_name);
                            
                            let tool_result = match function_name.as_str() {
                                "no_op" => {
                                    log!(info "Bot"; "Bot {} chose to do nothing", self.player_name);
                                    "No action taken".to_string()
                                }
                                "send_chat_message" => {
                                    if let Ok(args) = serde_json::from_str::<serde_json::Value>(arguments) {
                                        if let Some(message) = args.get("message").and_then(|m| m.as_str()) {
                                            // Find the chat controller
                                            let state = self.game_state.lock().await;
                                            if let Some(player_index) = state.player_index {
                                                let player = unsafe { PlayerReference::new_unchecked(player_index) };
                                                let _ = self.controller_sender.send(ControllerInput::new(
                                                    ControllerID::Chat { player },
                                                    crate::game::controllers::StringSelection(message.to_string())
                                                ));
                                                let _ = self.controller_sender.send(ControllerInput::new(
                                                    ControllerID::SendChat { player },
                                                    crate::game::controllers::UnitSelection
                                                ));
                                                format!("Sent chat message: {}", message)
                                            } else {
                                                "Failed to send chat: no player index".to_string()
                                            }
                                        } else {
                                            "Failed to send chat: invalid message".to_string()
                                        }
                                    } else {
                                        "Failed to send chat: invalid arguments".to_string()
                                    }
                                }
                                "send_ability_input" => {
                                    // Deserialize the controller input directly from the arguments
                                    match serde_json::from_str::<ControllerInput>(arguments) {
                                        Ok(controller_input) => {
                                            log!(info "Bot"; "Bot {} sending controller input: {:?}", self.player_name, controller_input);
                                            let _ = self.controller_sender.send(controller_input);
                                            "Sent ability input".to_string()
                                        }
                                        Err(e) => {
                                            format!("Failed to deserialize controller input: {}. Args: {}", e, arguments)
                                        }
                                    }
                                }
                                _ => {
                                    format!("Unknown tool: {}", function_name)
                                }
                            };

                            // Add tool result to conversation history
                            if let Ok(tool_message) = ChatCompletionRequestToolMessageArgs::default()
                                .content(tool_result)
                                .tool_call_id(tool_call_id)
                                .build()
                            {
                                self.conversation_history.push(ChatCompletionRequestMessage::Tool(tool_message));
                            }
                        }
                    } else if let Some(content) = &choice.message.content {
                        log!(info "Bot"; "Bot {} thinking: {}", self.player_name, content);
                    }
                }
            }
            Err(e) => {
                eprintln!("Bot {} LLM error: {}", self.player_name, e);
            }
        }

        self.last_decision_time = Some(Instant::now());
        Ok(())
    }
}
