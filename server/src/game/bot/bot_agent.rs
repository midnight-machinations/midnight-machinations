use std::sync::Arc;
use std::time::{Duration, Instant};
use async_openai::types::responses::{CreateResponseArgs, FunctionArgs, Input, InputContent, InputItem, InputMessage, InputMessageType, OutputContent, Role, ToolChoice, ToolChoiceMode, ToolDefinition};
use rand::Rng;
use tokio::sync::{mpsc, Mutex};
use async_openai::{
    Client,
};
use serde_json::json;

use crate::game::player::PlayerIndex;
use crate::{
    game::{controllers::{ControllerID, ControllerInput}, player::PlayerReference}, log, packet::ToClientPacket, room::RoomClientID,
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
    player_index: Option<PlayerIndex>
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
            const MIN_DELAY_SECS: u64 = 1;

            let mut decision_interval = tokio::time::interval(Duration::from_secs(MIN_DELAY_SECS));
            decision_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                tokio::select! {
                    Some(packet) = self.receiver.recv() => {
                        if let Err(e) = self.process_packet(packet).await {
                            log!(error "Bot"; "{} error processing packet: {}", self.player_name, e);
                        }
                    }
                    _ = decision_interval.tick() => {
                        // Time-based decision making
                        if let Err(e) = self.make_decision().await {
                            log!(error "Bot"; "{} error making decision: {}", self.player_name, e);
                        }

                        // Let the bot decide to wait if it desires.
                        decision_interval.reset_after(
                            self.desired_thought_interval
                                // Throw in some variability, so not all the bots talk at the same time.
                                .saturating_add(Duration::from_millis(rand::rng().random_range(0..=2000)))
                                .saturating_sub(Duration::from_millis(1000))
                                // Already waiting these 5 secs, see above
                                .saturating_sub(Duration::from_secs(MIN_DELAY_SECS))
                        );

                        self.desired_thought_interval = Duration::ZERO;
                    }
                }
            }
        })
    }

    fn translate(lang_key: &str) -> String {
        match serde_json::from_str::<serde_json::Value>(EN_US) {
            Ok(json) => {
                if let Some(desc) = json.pointer(&format!("/{lang_key}")) {
                    desc.as_str().unwrap_or("No description").to_string()
                } else {
                    log!(error "Bot"; "Description for {} not found in en_us.json", lang_key);
                    "No description".to_string()
                }
            }
            Err(_) => {
                log!(error "Bot"; "Failed to parse en_us.json");
                "Failed to read".to_string()
            },
        }
    }

    async fn process_packet(&mut self, packet: ToClientPacket) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.game_state.lock().await;

        match packet {
            ToClientPacket::YourPlayerIndex { player_index } => {
                state.player_index = Some(player_index);

                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(format!("Your player number is {} (player index {}).", player_index.saturating_add(1), player_index)),
                }));
            }
            ToClientPacket::YourRole { role } => {
                let Ok(role_name) = serde_json::to_string(&role) else {
                    return Ok(());
                };

                let role_name = role_name.trim_matches('"');

                // Provide context about the role from the wiki
                let role = Self::translate(&format!("role.{role_name}.name"));
                let reminder = Self::translate(&format!("wiki.article.role.{role_name}.reminder"));
                let guide = Self::translate(&format!("wiki.article.role.{role_name}.guide"));
                let abilities = Self::translate(&format!("wiki.article.role.{role_name}.abilities"));
                let attributes = Self::translate(&format!("wiki.article.role.{role_name}.attributes"));
                let extra = Self::translate(&format!("wiki.article.role.{role_name}.extra"));

                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(format!(
                        "Your role is now {role} This is its wiki page:\n\n\
                        Reminder (wiki.article.role.{role_name}.reminder):\n{reminder}\n\n\
                        Guide (wiki.article.role.{role_name}.guide):\n{guide}\n\n\
                        Abilities (wiki.article.role.{role_name}.abilities):\n{abilities}\n\n\
                        Attributes (wiki.article.role.{role_name}.attributes):\n{attributes}\n\n\
                        Extra (wiki.article.role.{role_name}.extra):\n{extra}"
                    )),
                }));
            }
            ToClientPacket::PhaseTimeLeft { seconds_left } => {
                match seconds_left {
                    Some(seconds_left) => {
                        self.queued_messages.push(InputItem::Message(InputMessage {
                            kind: InputMessageType::Message,
                            role: Role::User,
                            content: InputContent::TextInput(format!(
                                "This phase is {seconds_left} seconds long."
                            )),
                        }));
                    }
                    None => {
                        self.queued_messages.push(InputItem::Message(InputMessage {
                            kind: InputMessageType::Message,
                            role: Role::User,
                            content: InputContent::TextInput("This phase is indefinite.".to_string()),
                        }));
                    }
                }
            }
            ToClientPacket::Phase { phase, .. } => {
                let Ok(phase_name) = serde_json::to_string(&phase.phase()) else {
                    return Ok(());
                };

                let phase_name = phase_name.trim_matches('"');

                // Provide context about the current phase from the wiki
                let phase_title = Self::translate(&format!("wiki.article.standard.{phase_name}.title"));
                
                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(format!("The game phase has changed to {phase_title}.")),
                }));
            }

            ToClientPacket::AddChatMessages { chat_messages } => {
                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(format!(
                        "New chat message(s): {}",
                        serde_json::to_string_pretty(&chat_messages.values().collect::<Vec<_>>())?
                    )),
                }));
            }

            ToClientPacket::YourAllowedControllers { save } => {
                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(format!(
                        "Your available abilities changed. These are your available abilities: {}",
                        serde_json::to_string_pretty(&save)?
                    )),
                }));
            }
            ToClientPacket::YourAllowedController { id, controller } => {
                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(format!(
                        "Your available abilities for ability ID {} changed. These are your available abilities for ability ID {}: {}",
                        serde_json::to_string(&id)?,
                        serde_json::to_string(&id)?,
                        serde_json::to_string_pretty(&controller)?
                    )),
                }));
            }
            ToClientPacket::YourSendChatGroups { send_chat_groups } => {
                self.queued_messages.push(InputItem::Message(InputMessage {
                    kind: InputMessageType::Message,
                    role: Role::User,
                    content: InputContent::TextInput(format!("These are your chat groups: {send_chat_groups:?}.")),
                }));
            }
            _ => {}
        }

        Ok(())
    }

    async fn make_decision(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Define tools for the bot to use
        let tools = vec![
            ToolDefinition::Function(FunctionArgs::default()
                .name("wait")
                .description(
                    "Do nothing for a specified duration. Use of this tool is generally not a good idea unless you are \
                    certain you know everything you need to know to make your next move. The time waited is approximate, \
                    and may be more or less than the time specified."
                )
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
                .description(
                    "Send a chat message to other players. To mention someone in your message, use @<player_number> where \
                    <player_number> is their player index + 1. Chat messages you send will only be received by players in \
                    your current chat groups."
                )
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to send."
                        }
                    },
                    "required": ["message"]
                }))
                .build()?,
            ),
            ToolDefinition::Function(FunctionArgs::default()
                .name("send_ability_input")
                .description(
                    "Perform a game action by sending an ability input object with 'id' and 'selection' fields.\n\n\
                    `id` format:\n\
                    - For role abilities: {{\"type\": \"role\", \"player\": <your player_index>, \"role\": \"<roleName>\", \"id\": <ability_id>}}\n\
                    - For nomination: {{\"type\": \"nominate\", \"player\": <your player_index>}}\n\
                    - For judgement: {{\"type\": \"judge\", \"player\": <your player_index>}}\n\
                    - etc.\n\n\
                    `selection` format (choose based on ability type):\n\
                    - Unit: {{\"type\": \"unit\", \"selection\": null}}\n\
                    - Boolean: {{\"type\": \"boolean\", \"selection\": true/false}}\n\
                    - PlayerList: {{\"type\": \"playerList\", \"selection\": [player_indices]}}\n\
                    - String: {{\"type\": \"string\", \"selection\": \"text\"}}\n\
                    - Integer: {{\"type\": \"integer\", \"selection\": number}}\n\
                    - etc.".to_string()
                )
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "object",
                            "description": "The ability ID"
                        },
                        "selection": {
                            "type": "object",
                            "description": "The ability selection"
                        }
                    },
                    "required": ["id", "selection"]
                }))
                .build()?,
            ),
            ToolDefinition::Function(FunctionArgs::default()
                .name("read_wiki_article")
                .description("Read a wiki article or lang entry. The input should be a language key.")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "articleLink": {
                            "type": "string",
                            "description": "The link to the wiki article or lang entry to read."
                        }
                    },
                    "required": ["articleLink"]
                }))
                .build()?,
            ),
        ];

        let mut request_builder = CreateResponseArgs::default();
        
        // TODO: Determine best model to use
        request_builder.model("gpt-5-mini");

        if let Some(last_response) = &self.last_response {
            request_builder.previous_response_id(last_response);
        } else {
            let lang_keys = match serde_json::from_str::<serde_json::Value>(EN_US) {
                    Ok(json) => json.as_object()
                        .map(|obj| {
                            obj.keys()
                                .filter(|key| key.starts_with("wiki."))
                                .cloned()
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_default(),
                    Err(_) => Vec::new(),
                };

            log!(info "Bot"; "{lang_keys:#?}");

            request_builder.instructions(
                "# Identity

                You are a computer player in a social deduction game called Midnight Machinations. You are designed to
                play the game in lieu of a human player, making decisions based on the game state and chat messages.
                In order to do this, you will make strategic decisions using the tools provided to you.

                # Instructions

                ## Sending Chat Messages (send_chat_message tool)

                * You should use chat messages to communicate with other players, share information, and coordinate actions
                * Other players can't see the words you are saying unless you send chat messages using the appropriate tool.
                * Messages you send will be visible to players in every chat group you are in, and ONLY to those players. 
                  If you aren't in any chat groups, nobody can hear you! Take that time to read up on the wiki or use your abilities instead.
                  Also, not every chat group has other players in it.
                * If you are an evil role, you should make sure what you are saying doesn't give away your role or team.
                * Evil players should pretend to be good, and should claim a town role.
                * You cannot \"speak things into existence\". If you want to nominate someone or use your ability, you must
                  actually send an ability input, which is done using the send_ability_input tool.
                * While sharing your info is important, it's far more important to listen and respond to what other players
                  say. If someone claims to be a role, you should read that role's wiki article so you know what kind of
                  information they can get, and what plans you can make to figure out who the evil players are.
                
                ## Staying informed (read_wiki_article tool)

                * You should take the time to do this any time you are confused about events that have occured.
                * You should especially read the How To Play article at the start of the game to understand the rules and objectives.
                * You need to read the role sets article, otherwise you have no way of knowing whether your role is a good role
                  or an evil role.
                * Most games start with a briefing phase where players can read their roles and abilities and cannot
                  interact or send messages. You should spend as much of this time as possible reading about the game.
                  This is your opportunity as a computer player to try to get on-par with the more experienced players in the game.
                * As a computer player, you should make sure to read the wiki often so you can be just as informed as
                  the human players in the game who may have years of experience.
                * You should not blindly trust people. Evil players can (and should) claim to be good roles to gain trust.

                ## Making Game Actions (send_ability_input tool)

                * You should use game actions to further your role's objectives and help your team win the game.
                * You should carefully consider the game state and chat messages before making any game actions.
                * Using the send_ability_input tool is the *only way* to input your abilities.

                ## Other important things to know

                * Most games have 50% + 1 town players and the rest are syndicate or minion or fiend.
                * Whichever team has majority is the one with the power to vote people to execute at trial.
                * Waiting until a phase ends is not recommended ever. Instead, you should use your fast forward
                  ability to skip ahead to the next phase when you have nothing more to contribute. When everybody selects
                  to fast forward, the next phase will start immediately. If others aren't using their fast forward ability,
                  you should ask them to.
                * Syndicate, Fiends, Cult, Minions, Neutral, or anything else that isn't town are the evil teams.

                # Examples
                
                ## Dusk 1 Gossip Claim

                During dusk, a player claims that they are the gossip. As a responsible computer player, you read the 
                Gossip wiki article, which informs you that the gossip learns whether their target visited a friend of town.
                Because you know this, you can decide whether or not you should tell the gossip your role (or a false role)
                so they know whether they can visit you to get information (since not every role visits).

                ## Discussion Philosopher Claim

                During discussion, player 2 claims that they are a Philosopher who found player 3 and player 4 to be 
                enemies. If you don't already know how Philosopher works, you read the philosopher wiki and find out that
                the philosopher learns whether their targets are friends or enemies. You now know that at least one of 
                player 2, player 3, or player 4 are lying."
            );

            self.queued_messages.push(InputItem::Message(InputMessage {
                kind: InputMessageType::Message,
                role: Role::User,
                content: InputContent::TextInput(format!(
                    "Here is a list of every wiki key, for the read_wiki_article tool: {lang_keys:#?}"
                )),
            }));
        }

        let request = request_builder
            .input(Input::Items(self.queued_messages.drain(..).collect()))
            .tools(tools)
            .tool_choice(ToolChoice::Mode(ToolChoiceMode::Required))
            .max_output_tokens(25_000_u32)
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
                                        log!(info "Bot"; "Bot \"{}\" chose to wait {} seconds", self.player_name, self.desired_thought_interval.as_secs());
                                    }
                                }
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
                                        log!(info "Bot"; "Bot \"{}\" sending ability input: {:?}", self.player_name, controller_input);
                                        let _ = self.controller_sender.send(controller_input);
                                        "Sending ability input\n\n\
                                        If a confirmation of your ability input doesn't appear in chat:\n\
                                        * It likely failed.\n\
                                        * You may want to attempt to re-send it.\n\
                                        * You may have to wait until your ability is usable.\
                                        ".to_string()
                                    }
                                    Err(e) => {
                                        format!("Failed to deserialize ability input: {e}. Args: {arguments}")
                                    }
                                }
                            }
                            "read_wiki_article" => {
                                if let Ok(args) = serde_json::from_str::<serde_json::Value>(arguments) {
                                    if let Some(article_link) = args.get("articleLink").and_then(|m| m.as_str()) {
                                        log!(info "Bot"; "Bot \"{}\" reading wiki article: {:?}", self.player_name, article_link);
                                        if article_link == "wiki.article.generate.roleSet.text" {
"
# Role Sets
## Town

Armorsmith, Auditor, Bodyguard, Bouncer, Cop, Courtesan, Deputy, Detective, Doctor, Drunk, Engineer, Escort, Gossip, Jailor, Lookout, Marksman, Mayor, Medium, Nepotist, Philosopher, Polymath, Porter, Psychic, Rabblerouser, Reporter, Snoop, Spy, Steward, Tally Clerk, Tracker, Transporter, Veteran, Vigilante, Villager

## Town Common

All Town roles except Jailor, Villager, and Drunk.

Armorsmith, Auditor, Bodyguard, Bouncer, Cop, Courtesan, Deputy, Detective, Doctor, Engineer, Escort, Gossip, Lookout, Marksman, Mayor, Medium, Nepotist, Philosopher, Polymath, Porter, Psychic, Rabblerouser, Reporter, Snoop, Spy, Steward, Tally Clerk, Tracker, Transporter, Veteran, Vigilante

## Town Investigative

Auditor, Detective, Gossip, Lookout, Philosopher, Psychic, Snoop, Spy, Tally Clerk, Tracker

## Town Protective

Armorsmith, Bodyguard, Bouncer, Cop, Doctor, Engineer, Steward

## Town Killing

Deputy, Marksman, Rabblerouser, Veteran, Vigilante

## Town Support

Courtesan, Escort, Mayor, Medium, Nepotist, Polymath, Porter, Reporter, Transporter

## Syndicate

Ambusher, Blackmailer, Cerenovous, Consort, Counterfeiter, Disguiser, Forger, Framer, Godfather, Goon, Hypnotist, Impostor, Informant, Mafioso, Maverick, Mortician, Necromancer, Recruiter, Reeducator, Syndicate Killing Wildcard, Syndicate Support Wildcard

## Syndicate Killing

Counterfeiter, Godfather, Impostor, Mafioso, Necromancer, Recruiter

## Syndicate Support

Ambusher, Blackmailer, Cerenovous, Consort, Disguiser, Forger, Framer, Hypnotist, Informant, Mortician, Reeducator

## Neutral

Chronokaiser, Jester, Krampus, Martyr, Mercenary, Politician, Revolutionary, Santa Claus, True Wildcard, Wildcard

## Minions

Kidnapper, Pawn, Scarecrow, Tailor, Warper, Witch

## Fiends

Arsonist, Fiends Wildcard, Juggernaut, Kira, Ojo, Puppeteer, Pyrolisk, Serial Killer, UZUMAKI, Warden, Werewolf, Yer

## Cult

Apostle, Disciple, Zealot

There are a total of 86 roles.".to_string() } else {
                                        Self::translate(article_link)
                                        }
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
