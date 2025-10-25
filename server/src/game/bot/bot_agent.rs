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
    game::controllers::ControllerInput,
    packet::ToClientPacket,
    room::RoomClientID,
};

/// Bot agent that uses an LLM to make decisions in the game
pub struct BotAgent {
    #[allow(dead_code, reason = "Will be used for sending actions in future implementation")]
    player_id: RoomClientID,
    player_name: String,
    receiver: mpsc::UnboundedReceiver<ToClientPacket>,
    #[allow(dead_code, reason = "Will be used for sending controller inputs in future implementation")]
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
            ToClientPacket::YourAllowedControllers { .. } => {
                // When we get new controllers, we might want to use them
                drop(state);
                self.make_decision().await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn make_decision(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // This is a simplified decision-making process
        // In a real implementation, this would be more sophisticated
        
        let state = self.game_state.lock().await;
        
        // Build context for the LLM
        let context = format!(
            "You are playing a social deduction game called Midnight Machinations.\n\
             Your name: {}\n\
             Your role: {}\n\
             Current phase: {}\n\
             Alive players: {:?}\n\
             Recent messages: {:?}\n\n\
             Based on this information, decide what action to take. \
             Respond with a brief explanation of your reasoning.",
            self.player_name,
            state.role.as_ref().unwrap_or(&"Unknown".to_string()),
            state.phase.as_ref().unwrap_or(&"Unknown".to_string()),
            state.alive_status,
            state.recent_messages
        );

        drop(state);

        // Query the LLM (with error handling for API calls)
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are an AI playing a social deduction game. Make strategic decisions based on the game state.")
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
            .max_tokens(150_u32)
            .build()?;

        // Try to get a response from the LLM
        match self.openai_client.chat().create(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first()
                && let Some(content) = &choice.message.content {
                    println!("Bot {} thinking: {}", self.player_name, content);
                }
            }
            Err(e) => {
                eprintln!("Bot {} LLM error: {}", self.player_name, e);
            }
        }

        // TODO: Parse LLM response and generate ControllerInput
        // For now, bots will just observe without acting

        Ok(())
    }
}
