# Bot Player System

This module implements LLM-powered bot players for Midnight Machinations.

## Overview

The bot player system allows hosts to add AI-controlled players to their games. These bots use OpenAI's ChatGPT API to make decisions based on the game state and send controller inputs to interact with the game.

## Architecture

- **BotConnection**: A channel-based connection type that receives game packets
- **BotAgent**: The agent that processes game state and uses an LLM to make decisions
- **ClientConnection::Bot**: A variant of the ClientConnection enum specifically for bot players
- **Bot Thread Management**: Bot agents run in separate tokio tasks and send controller inputs back to the game

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
6. The agent uses the LLM to decide on actions based on:
   - Current role
   - Phase of the game
   - Player alive status
   - Recent chat messages
   - Available controllers/abilities
7. Bot decisions are sent back through a controller input channel
8. The game processes bot inputs in its tick function

## Features Implemented

✅ **Controller Input Generation**: Bots analyze available controllers and choose actions
✅ **Chat Participation**: Bots can send chat messages when they have chat controllers
✅ **Bot Thread Management**: Bot agents are properly spawned and managed throughout game lifecycle
✅ **Automatic Cleanup**: Bot threads are cleaned up when the game ends

## Bot Decision Making

Bots use a simplified decision-making process:
1. Receive available controllers from the game
2. Build context with game state (role, phase, recent messages)
3. Query ChatGPT with available actions
4. Parse LLM response (either a controller index or "CHAT: message")
5. Send appropriate controller input back to game

## Current Limitations

- Bots use current controller selections rather than choosing specific targets
- Decision making is relatively simple (could be enhanced with more sophisticated prompts)
- LLM API calls may introduce latency
- Bots don't have long-term memory across phases

## Future Improvements

- [ ] Enhance target selection (e.g., bots choosing specific players to target)
- [ ] Add bot memory system for better strategic play
- [ ] Implement caching to reduce API calls
- [ ] Add configurable bot difficulty levels
- [ ] Support for different LLM models
- [ ] Batch processing of bot decisions
- [ ] More sophisticated chat participation
