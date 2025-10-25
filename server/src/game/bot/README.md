# Bot Player System

This module implements LLM-powered bot players for Midnight Machinations.

## Overview

The bot player system allows hosts to add AI-controlled players to their games. These bots use OpenAI's ChatGPT API to make decisions based on the game state.

## Architecture

- **BotConnection**: A channel-based connection type that receives game packets
- **BotAgent**: The agent that processes game state and uses an LLM to make decisions
- **ClientConnection::Bot**: A variant of the ClientConnection enum specifically for bot players

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
4. When the game starts, a BotAgent thread is spawned for each bot
5. The BotAgent receives game state updates via packets
6. The agent uses the LLM to decide on actions based on:
   - Current role
   - Phase of the game
   - Player alive status
   - Recent chat messages
   - Available controllers/abilities

## Current Limitations

- Bots currently observe the game but don't take actions (decision-making is partially implemented)
- Bot decision-making needs to be enhanced to properly parse LLM responses and generate ControllerInput
- Bots don't participate in chat
- LLM API calls may introduce latency

## Future Improvements

- [ ] Complete controller input generation from LLM responses
- [ ] Add bot chat participation
- [ ] Implement caching to reduce API calls
- [ ] Add configurable bot difficulty levels
- [ ] Support for different LLM models
- [ ] Batch processing of bot decisions
