use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use async_openai::{
    Client,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
};

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

        // Build a list of available actions
        let available_actions: Vec<String> = state.available_controllers
            .iter()
            .map(|(id, controller)| {
                format!("Controller: {:?}, Current selection: {:?}", id, controller.selection())
            })
            .collect();

        if available_actions.is_empty() {
            return Ok(());
        }
        
        // Build context for the LLM
        let context = format!(
            "You are playing a social deduction game called Midnight Machinations.\n\
             Your name: {}\n\
             Your role: {}\n\
             Current phase: {}\n\
             Alive players: {:?}\n\
             Recent messages: {}\n\
             Available actions: {}\n\n\
             Choose ONE action to take from the available actions. \
             Respond with ONLY the controller index (0-{}) you want to use, or 'CHAT: <message>' to send a chat message. \
             Be strategic and consider your role's objectives.",
            self.player_name,
            state.role.as_ref().unwrap_or(&"Unknown".to_string()),
            state.phase.as_ref().unwrap_or(&"Unknown".to_string()),
            state.alive_status,
            state.recent_messages.last().unwrap_or(&"None".to_string()),
            available_actions.join(", "),
            available_actions.len().saturating_sub(1)
        );

        let controllers_list: Vec<(ControllerID, Controller)> = state.available_controllers
            .iter()
            .map(|(id, c)| (id.clone(), c.clone()))
            .collect();

        drop(state);

        // Query the LLM
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are an AI playing a social deduction game. Make strategic decisions. Respond with only a number or 'CHAT: message'.")
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
            .max_tokens(100_u32)
            .temperature(0.7)
            .build()?;

        // Try to get a response from the LLM
        match self.openai_client.chat().create(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first()
                && let Some(content) = &choice.message.content {
                    let content = content.trim();
                    println!("Bot {} decided: {}", self.player_name, content);
                    
                    // Parse the response
                    if content.starts_with("CHAT:") {
                        // Bot wants to send a chat message
                        if let Some(message) = content.strip_prefix("CHAT:") {
                            let message = message.trim();
                            if !message.is_empty() {
                                // Find the chat controller
                                for (id, _) in &controllers_list {
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
                    } else if let Ok(index) = content.parse::<usize>() {
                        // Bot chose a controller by index
                        if index < controllers_list.len() {
                            let (controller_id, controller) = &controllers_list[index];
                            
                            // Use the current selection (bots keep existing selections for simplicity)
                            // In a more sophisticated implementation, we'd have the LLM choose specific targets
                            let input = ControllerInput::new(
                                controller_id.clone(),
                                controller.selection().clone()
                            );
                            
                            let _ = self.controller_sender.send(input);
                            println!("Bot {} sent controller input: {:?}", self.player_name, controller_id);
                        }
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
