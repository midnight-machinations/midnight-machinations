# Bot Player System

This module implements LLM-powered bot players for Midnight Machinations.

## Overview

The bot player system allows hosts to add AI-controlled players to their games. These bots use OpenAI's ChatGPT API with function calling (tool calls) to make decisions and interact with the game. Bots maintain conversation history for context-aware decision making and use a time-based approach to avoid excessive API calls.

## Architecture

- **BotConnection**: A channel-based connection type that receives game packets
- **BotAgent**: The agent that processes game state and uses an LLM to make decisions
- **ClientConnection::Bot**: A variant of the ClientConnection enum specifically for bot players
- **Bot Thread Management**: Bot agents run in separate tokio tasks and send controller inputs back to the game
- **Tool Calling**: Bots use OpenAI's function calling feature to send structured actions
- **Conversation History**: Bots maintain context across multiple interactions
- **Time-based Decision Making**: Bots make decisions every ~7 seconds instead of on every event

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
6. Every ~7 seconds, the bot makes a decision:
   - Reviews accumulated game state changes
   - Queries ChatGPT with conversation history
   - Receives tool calls from the LLM
   - Executes actions or chooses no-op
7. Tool responses are added to conversation history for future context
8. The game processes bot inputs in its tick function

## Tool Calling

Bots use OpenAI's function calling feature with three tools:

### no_op
Do nothing. Used when the bot has nothing to say or do at the moment.

**Parameters:** None

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
✅ **Conversation History**: Bots maintain context across multiple interactions (last 20 messages + system prompt)
✅ **Time-based Decision Making**: Bots make decisions every ~7 seconds, reducing API calls
✅ **No-op Support**: Bots can choose to do nothing when appropriate

## Bot Decision Making

Bots use OpenAI's function calling with conversation history:
1. Receive available controllers from the game (serialized to JSON)
2. Build context with recent game state changes (last 5 chat messages, phase, alive status)
3. Query ChatGPT with tool definitions and conversation history
4. LLM chooses which tool to call based on context and generates structured arguments
5. Bot agent executes tool calls and adds results to conversation history
6. Process repeats every ~7 seconds

## Advantages

- **Conversation History**: Bots remember previous interactions and build on them
- **Reduced API Calls**: Time-based approach (~7s intervals) instead of event-driven
- **Context Awareness**: Bots can reference earlier decisions and information
- **Strategic Play**: Better decision making with historical context
- **No-op Option**: Bots can choose to wait when there's nothing to do
- **Structured Output**: No text parsing needed, direct JSON deserialization
- **Type Safety**: LLM generates JSON that matches ControllerInput structure
- **Better Accuracy**: LLM understands the structure of actions
- **Easier Debugging**: Tool calls are logged with arguments
- **Extensible**: Easy to add new tools for different actions

## Current Limitations

- LLM may not always choose optimal targets
- Conversation history limited to last 20 messages to manage token usage
- LLM API calls may introduce latency (mitigated by time-based approach)
- Tool calling requires OpenAI models with function support (gpt-4o-mini, gpt-4, etc.)
- 7-second decision interval may feel slow during fast-paced phases

## Future Improvements

- [ ] Configurable decision interval based on game phase
- [ ] More sophisticated memory system (summarization, key facts extraction)
- [ ] Implement caching to reduce API calls
- [ ] Add configurable bot difficulty levels
- [ ] Support for different LLM models
- [ ] Batch processing of bot decisions
- [ ] More sophisticated game state representation
- [ ] Additional tools for specific game actions
- [ ] Personality traits for different bot behaviors
