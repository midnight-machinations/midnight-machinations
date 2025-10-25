use std::sync::Arc;
use std::time::{Duration, Instant};
use async_openai::types::responses::{CreateResponseArgs, FunctionArgs, Input, InputContent, InputItem, InputMessage, InputMessageType, OutputContent, Role, ToolChoice, ToolChoiceMode, ToolDefinition};
use tokio::sync::{mpsc, Mutex};
use async_openai::{
    Client,
};
use serde_json::json;

use crate::game::chat::ChatGroup;
use crate::vec_set::VecSet;
use crate::{
    game::{controllers::{Controller, ControllerID, ControllerInput}, player::PlayerReference}, log, packet::ToClientPacket, room::RoomClientID, vec_map::VecMap
};

/// Bot agent that uses an LLM to make decisions in the game
pub struct BotAgent {
    #[expect(unused, reason = "We might need it someday")]
    player_id: RoomClientID,
    player_name: String,
    receiver: mpsc::UnboundedReceiver<ToClientPacket>,
    controller_sender: mpsc::UnboundedSender<ControllerInput>,
    openai_client: Client<async_openai::config::OpenAIConfig>,
    game_state: Arc<Mutex<BotGameState>>,
    last_response: Option<String>,
    queued_messages: Vec<InputItem>,
    last_decision_time: Option<Instant>,
    desired_thought_interval: Duration,
}

#[derive(Default)]
struct BotGameState {
    role: Option<String>,
    phase: Option<String>,
    alive_status: Vec<bool>,
    recent_messages: Vec<String>,
    send_chat_groups: VecSet<ChatGroup>,
    available_controllers: VecMap<ControllerID, Controller>,
    player_index: Option<u8>,
}

static EN_US: &str = include_str!("../../../../client/src/resources/lang/en_us.json");

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
            last_response: None,
            queued_messages: vec![],
            last_decision_time: None,
            desired_thought_interval: Duration::ZERO,
        }
    }

    /// Start the bot agent in a separate thread
    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut decision_interval = tokio::time::interval(Duration::from_secs(5).max(self.desired_thought_interval));
            decision_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            self.desired_thought_interval = Duration::ZERO;
            
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

    fn call_upon_the_oracle(wiki_key: &str, what_am_i_talking_about: &str,) -> String {
        let wiki_content = {
            match serde_json::from_str::<serde_json::Value>(EN_US) {
                Ok(json) => {
                    let key = wiki_key.to_lowercase();
                    if let Some(desc) = json.pointer(&format!("/{key}")) {
                        desc.as_str().unwrap_or("No description available").to_string()
                    } else {
                        log!(error "Bot"; "Description for {} not found in en_us.json", wiki_key);
                        "No description available - Wiki Page does not exist".to_string()
                    }
                }
                Err(_) => {
                    log!(error "Bot"; "Failed to parse en_us.json");
                    "No description available".to_string()
                },
            }
        };
        
        format!("Here is the description of {what_am_i_talking_about} from the wiki:\n{wiki_content}")
    }

    async fn process_packet(&mut self, packet: ToClientPacket) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.game_state.lock().await;

        match packet {
            ToClientPacket::YourPlayerIndex { player_index } => {
                state.player_index = Some(player_index);
            }
            ToClientPacket::YourRole { role } => {
                state.role = Some(format!("{role:?}"));

                let Ok(role_name) = serde_json::to_string(&role) else {
                    return Ok(());
                };

                let role_name = role_name.trim_matches('"');

                // Provide context about the role from the wiki
                let message = Self::call_upon_the_oracle(
                    &format!("wiki.article.role.{role_name}.abilities"),
                    "your role and its abilities",
                );

                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(message),
                }));
            }
            ToClientPacket::Phase { phase, .. } => {
                state.phase = Some(format!("{phase:?}"));

                let Ok(phase_name) = serde_json::to_string(&phase.phase()) else {
                    return Ok(());
                };

                let phase_name = phase_name.trim_matches('"');

                // Provide context about the current phase from the wiki
                let message = Self::call_upon_the_oracle(
                    &format!("wiki.article.standard.{phase_name}.text"),
                    "the current phase",
                );
                
                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(message),
                }));
            }
            ToClientPacket::PlayerAlive { alive } => {
                state.alive_status = alive;
            }
            ToClientPacket::AddChatMessages { chat_messages } => {
                for (_, msg) in chat_messages.iter() {
                    let msg_str = format!("{msg:?}");
                    state.recent_messages.push(msg_str);
                    if state.recent_messages.len() > 100 {
                        let num_messages = state.recent_messages.len();

                        #[expect(clippy::arithmetic_side_effects, reason = "Checked manually")]
                        state.recent_messages.drain(0..(num_messages - 100));
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
            ToClientPacket::YourSendChatGroups { send_chat_groups } => {
                state.send_chat_groups = send_chat_groups;
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
             Players alive: {:?}\n\
             Recent messages: {:?}\n\
             You can talk in these chat groups: {:?}\n\n\
             You have controllers available to send actions. Consider your role's objectives and make strategic decisions.\n\
             Use the send_ability_input tool to perform game actions, send_chat_message to communicate with other players, or wait if you have nothing to do right now. \
             You can also use the read_wiki_article tool to look up information on the game's wiki if needed.",
            self.player_name,
            state.alive_status,
            state.recent_messages.iter().rev().take(5).collect::<Vec<_>>(),
            state.send_chat_groups
        );

        // Serialize available controllers to show the bot what's available
        let controllers_json = serde_json::to_string_pretty(&state.available_controllers)?;

        drop(state);

        // Define tools for the bot to use
        let tools = vec![
            ToolDefinition::Function(FunctionArgs::default()
                .name("wait")
                .description("Do nothing. Use this when you have nothing to say or do at the moment, or when you're waiting for other players to act.")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "durationSeconds": {
                            "type": "integer",
                            "description": "Number of seconds to wait before considering your next action (Value must be greater than or equal to 5 seconds)."
                        }
                    },
                    "required": ["durationSeconds"]
                }))
                .build()?,
            ),
            ToolDefinition::Function(FunctionArgs::default()
                .name("send_chat_message")
                .description("Send a chat message to other players")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to send. To mention someone in your message, use @<player_index> where <player_index> is their player index + 1."
                        }
                    },
                    "required": ["message"]
                }))
                .build()?,
            ),
            ToolDefinition::Function(FunctionArgs::default()
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
                    Available controllers:\n{controllers_json}"
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
            ),
            ToolDefinition::Function(FunctionArgs::default()
                .name("read_wiki_article")
                .description("Read a wiki article")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "articleLink": {
                            "type": "string",
                            "description": "The link to the wiki article to read (e.g., \"wiki.article.role.detective.abilities\" or \"wiki.article.standard.aura.text\")."
                        }
                    },
                    "required": ["articleLink"]
                }))
                .build()?,
            ),
        ];

        let mut request_builder = CreateResponseArgs::default();
        
        // TODO: Determine best model to use
        request_builder.model("gpt-4o-mini");

        if let Some(last_response) = &self.last_response {
            request_builder.previous_response_id(last_response);
        } else {
            let how_to_play = Self::call_upon_the_oracle("wiki.article.standard.howToPlay.text", "how to play Midnight Machinations");

            request_builder.instructions(format!(
                "You are playing a social deduction game called Midnight Machinations. \
                Make strategic decisions using the provided tools. \
                You can send chat messages or ability inputs based on your role and the game situation. \
                If you have nothing to do or say at the moment, use the wait tool to indicate you're waiting.\n\
                Player indices display starting from 1, but are zero-indexed internally. You will see the zero indexes \
                in controller IDs and selections, but should index from 1 when mentioning players in chat messages.\n\n\
                {how_to_play}\n\n\
                Most games start with a briefing phase where players can read their roles and abilities and cannot interact. \
                If you find yourself in this briefing phase, you should peruse the wiki to better understand the game around you."
            ));
        }

        self.queued_messages.push(InputItem::Message(InputMessage {
            kind: InputMessageType::Message,
            role: Role::User,
            content: InputContent::TextInput(context),
        }));

        let request = request_builder
            .input(Input::Items(self.queued_messages.drain(..).collect()))
            .tools(tools)
            .tool_choice(ToolChoice::Mode(ToolChoiceMode::Required))
            .max_output_tokens(200_u32)
            .temperature(0.7)
            .build()?;

        // Try to get a response from the LLM
        match self.openai_client.responses().create(request).await {
            Ok(response) => {
                self.last_response = Some(response.id.clone());

                let mut outputs = response.output.into_iter();

                while outputs.len() > 0 {
                    if let Some(OutputContent::FunctionCall(function_call)) = outputs.next() {
                        let function_name = &function_call.name;
                        let arguments = &function_call.arguments;
                        let call_id = &function_call.call_id;
                        
                        log!(info "Bot"; "Bot \"{}\" calling tool: {}\n{}", self.player_name, function_name, arguments);
                        
                        let tool_result = match function_name.as_str() {
                            "wait" => {
                                if let Ok(args) = serde_json::from_str::<serde_json::Value>(arguments) {
                                    if let Some(duration) = args.get("durationSeconds").and_then(|d| d.as_u64()) {
                                        self.desired_thought_interval = Duration::from_secs(duration);
                                    }
                                }
                                log!(info "Bot"; "Bot {} chose to wait {} seconds", self.player_name, self.desired_thought_interval.as_secs());
                                "Waiting...".to_string()
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
                                            "Sending chat message.\n\n\
                                            If your message doesn't appear in chat:\n\
                                            * It likely failed to send.\n\
                                            * You may want to attempt to re-send it.\n\
                                            * You may have to wait until you are in a chat group.\
                                            ".to_string()
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
                                        "Sending ability input\n\n\
                                        If a confirmation of your ability input doesn't appear in chat:\n\
                                        * It likely failed.\n\
                                        * You may want to attempt to re-send it.\n\
                                        * You may have to wait until your ability is usable.\
                                        ".to_string()
                                    }
                                    Err(e) => {
                                        format!("Failed to deserialize controller input: {e}. Args: {arguments}")
                                    }
                                }
                            }
                            "read_wiki_article" => {
                                if let Ok(args) = serde_json::from_str::<serde_json::Value>(arguments) {
                                    if let Some(article_link) = args.get("articleLink").and_then(|m| m.as_str()) {
                                        Self::call_upon_the_oracle(article_link, "the requested wiki article")
                                    } else {
                                        "Failed to read wiki article: invalid articleLink".to_string()
                                    }
                                } else {
                                    "Failed to read wiki article: invalid arguments".to_string()
                                }
                            }
                            _ => {
                                format!("Unknown tool: {function_name}")
                            }
                        };

                        self.queued_messages.push(InputItem::Custom(json!({
                            "type": "function_call_output",
                            "call_id": call_id,
                            "output": tool_result,
                        })));
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
