# Bot Player System

This module implements LLM-powered bot players for Midnight Machinations.

## Overview

The bot player system allows hosts to add AI-controlled players to their games. These bots use OpenAI's ChatGPT API with function calling (tool calls) to make decisions and interact with the game.

## Architecture

- **BotConnection**: A channel-based connection type that receives game packets
- **BotAgent**: The agent that processes game state and uses an LLM to make decisions
- **ClientConnection::Bot**: A variant of the ClientConnection enum specifically for bot players
- **Bot Thread Management**: Bot agents run in separate tokio tasks and send controller inputs back to the game
- **Tool Calling**: Bots use OpenAI's function calling feature to send structured actions

## Configuration

To enable bots, you need to set the OpenAI API key as an environment variable:

```bash
export OPENAI_API_KEY="your-api-key-here"
```

Or add it to your `.env` file:

```
OPENAI_API_KEY=your-api-key-here
```

## How It Works

1. Host clicks "Add Bot" in the lobby
2. A bot player is created with a unique name (e.g., "Bot", "Bot 1", etc.)
3. A BotConnection is created with a channel to receive game packets
4. When the game starts, a BotAgent task is spawned for each bot
5. The BotAgent receives game state updates via packets (role, phase, controllers, chat messages)
6. The agent uses the LLM with tool calling to decide on actions based on:
   - Current role
   - Phase of the game
   - Player alive status
   - Recent chat messages
   - Available controllers/abilities
7. Bot decisions are made via tool calls:
   - `send_chat_message`: Send a chat message to other players
   - `send_ability_input`: Send a ControllerInput to perform game actions
8. The game processes bot inputs in its tick function

## Tool Calling

Bots use OpenAI's function calling feature with two tools:

### send_chat_message
Sends a chat message to other players.

**Parameters:**
```json
{
  "message": "string"
}
```

### send_ability_input
Sends a controller input to perform a game action. The input uses the same JSON format as the `ControllerInput` packet that clients send.

**Parameters:**
```json
{
  "id": {
    "type": "role|sendChat|nominate|judge|...",
    "player": 0,
    "role": "RoleName",
    "id": 0
  },
  "selection": {
    "type": "unit|boolean|playerList|string|integer|...",
    "selection": "value"
  }
}
```

The bot is provided with:
- Available controllers in JSON format
- Format documentation for ControllerID and ControllerSelection
- Context about the current game state

## Features Implemented

✅ **Tool-based Controller Input**: Bots use OpenAI function calling to generate structured ControllerInput
✅ **Chat Participation**: Bots can send chat messages using the send_chat_message tool
✅ **Bot Thread Management**: Bot agents are properly spawned and managed throughout game lifecycle
✅ **Automatic Cleanup**: Bot threads are cleaned up when the game ends
✅ **JSON Format**: Bots understand and generate ControllerInput in the same JSON format as clients

## Bot Decision Making

Bots use OpenAI's function calling:
1. Receive available controllers from the game (serialized to JSON)
2. Build context with game state (role, phase, recent messages)
3. Query ChatGPT with tool definitions
4. LLM chooses which tool to call and generates structured arguments
5. Bot agent deserializes tool call arguments and sends appropriate controller input

## Advantages of Tool Calling

- **Structured Output**: No text parsing needed, direct JSON deserialization
- **Type Safety**: LLM generates JSON that matches ControllerInput structure
- **Better Accuracy**: LLM understands the structure of actions
- **Easier Debugging**: Tool calls are logged with arguments
- **Extensible**: Easy to add new tools for different actions

## Current Limitations

- LLM may not always choose optimal targets
- No long-term memory across phases
- LLM API calls may introduce latency
- Tool calling requires OpenAI models with function support (gpt-4o-mini, gpt-4, etc.)

## Future Improvements

- [ ] Add memory system for better strategic play
- [ ] Implement caching to reduce API calls
- [ ] Add configurable bot difficulty levels
- [ ] Support for different LLM models
- [ ] Batch processing of bot decisions
- [ ] More sophisticated game state representation
- [ ] Additional tools for specific game actions
