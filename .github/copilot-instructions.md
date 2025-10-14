# GitHub Copilot Instructions

## Code Generation Guidelines

- Think through your solutions step by step.
- Feel free to ask clarifying questions along the way.
- When you ask a clarifying question, allow me to respond before continuing.
- Check for errors whenever you finish making code edits. If there are any, continue to iterate until you've resolved them.
- Don't assume the contents of other files. Check exports manually if you need to use something from another file.

## Adding New Roles

### Backend (`server/`)

* Create a file for the role in the roles folder named `server/src/game/role/<role>.rs`

#### In `server/src/game/role/mod.rs`

* Add the role to the `roles!` macro
* If the role is control immune or roleblock immune, add to those functions
* If the role has suspicious or innocent aura, add to those functions

#### Elsewhere:

* Add to a role set in the `RoleSet::get_roles` function
* Add associated `game::tag::Tag`s, `game::chat::ChatMessage`s, `packet::ToClientPacket`s, and `packet::ToServerPacket`s, if there are any.

### Frontend (`client/`)

* Add the role to the RoleState type in `client/src/game/roleState.d.tsx`
* Add the role to `client/src/resources/roles.json`
  - Remember to supply associated chat messages - this is for the wiki
* Handle any new chat messages in `client/src/components/ChatMessage.tsx` and `client/src/resources/styling/chatMessage.json` and add it roles in `client/src/resources/roles.json`
* Add tags
* Add any new packets to `client/src/game/packet.tsx` and `client/src/game/messageListener.tsx`
* Add language in `client/src/resources/lang/en_us.json`
  - Wiki page
  - Priority wiki page
  - The role itself
  - The role's controller names
  - Chat messages (if any)
  - Tags (If any)
* Create role-specific menu 
  - If no information is required, skip this step.
  - If only a small amount of information is required, create a small role-specific menu by editing the render function in `client/src/menu/game/gameScreenContent/AbilityMenu/RoleSpecific.tsx`.
  - If a large amount of information is required, create a large role-specific menu by creating a new file in `client/src/menu/game/gameScreenContent/AbilityMenu/RoleSpecificMenus/`.

## Creating new Messages

### Backend

#### Formatting

* The name of the message should be written in UpperCamelCase
* The fields of the message should be written in snake_case

#### In `server/src/game/chat/chat_message_variant.rs`:

* Add the message and its info as an enum variant.
* If your chat message contains fields with underscores, i.e. `player_ref`, you need to put `#[serde(rename_all = "camelCase")]` above its enum variant so the frontend receives the data in camelCase.

#### Misc:

* If the message is a message players should receive at the end of night, it should be sent to the player via `player_ref.push_night_message(game, ChatMessageVariant::<YourMessage>{<fields>})`.
* If the message is being sent to a [chat group](https://midnightmachinations.net/wiki/standard/chatGroup), it should be sent via `game.add_message_to_chat_group(<group>, ChatMessageVariant::<YourMessage>{<fields>})`. (Note All is a chat group).
* If the message is sent to individual players and is not an end of night message, it should be sent to the player via `player_ref.add_private_chat_message(game, ChatMessageVariant::<YourMessage>{<fields>})`.

### Frontend

#### Formatting

* The name of the message should be written in lowerCamelCase
* The fields of the message should be written in lowerCamelCase

#### In `client/src/resources/lang/en_us.json`:
* Add a key in the format `"chatMessage.<yourChatMessage>": "<Your message text.>"`.

#### In `client/src/components/ChatMessage.tsx`:
* Add the type and its corresponding fields to the `export type ChatMessageVariant` type.
* Add a case in the `export function translateChatMessage` function that translates the message.

#### In `client/src/resources/roles.json`
* If your message is related to specific role(s), add the message to the role's `.chatMessages` field.
* This will add the messages to its wiki page.

#### In `client/src/resources/styling/chatMessage.json`
* Add what style the message should be.
* **Styles**
  * `result`: The style used for messages containing things like TI, attacks and block information.
  * `special`: The style used typically used messages that update you on the current state of your abilities, e.g. the message at the beginning of each night that tells a engineer about the state of their trap.
  * `important`: The style of the messages that say when a player died and when a phase is fast forwarded.
  * `center`: Can be combined with other styles, will center the message
  * `discreet`: The style used for whispers. Not recommended.
  * `warning`: The style of the message that tells you that you died. Not recommended.
  * `target`: The style that is used for telling players and insiders what the player selected for their ability. Not recommended.
  * `trial`: The style of the player voted xyz messages use and the style of messages sent when game ends, listing whether a player won or lost
  * `phase-change`: The style that phase change messages use. Don't use this.
