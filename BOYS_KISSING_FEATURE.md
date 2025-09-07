# Boys Kissing Feature Documentation 💋👬

## Overview
This feature adds adorable boys kissing mechanics to the Mafia game! It includes visual animations, romantic phases, special roles, and customizable settings to make the game more fun and inclusive.

## Features Added

### 🎨 Components
- **BoysKissing.tsx**: Main animated component showing two boys sharing a romantic moment
- **BoysKissingOverlay.tsx**: Overlay that manages random romantic events during gameplay  
- **useBoysKissing.tsx**: React hook that handles boys kissing logic and events
- **RomanceModeSettings.tsx**: Settings panel for customizing romance features

### 💕 Language Additions
Added to `en_us.json`:
- Chat messages for different types of kisses (cute, romantic, wholesome, passionate)
- New role definitions (Cupid, Lovebird)
- Romantic phase descriptions
- Win condition messages for love-based victories
- Romance-themed icons and UI text

### 🌙 New Game Elements
- **Romantic Evening Phase**: Special phase where love is in the air
- **Cupid Role**: Can make players fall in love with kiss abilities
- **Lovebird Role**: Players who are matched by Cupid
- **Romance Win Conditions**: Special victory conditions for love-based gameplay

### 🎮 Game Integration
- Integrated into the main GameScreen component
- Random romantic events during gameplay
- Kiss trigger button for manual romantic moments
- Player statistics tracking (total kisses, favorite couples)

## How to Use

### Basic Usage
The boys kissing feature is automatically enabled in the game. Players will see:
- A "💋 Kiss" button in the top-right corner during gameplay
- Random romantic moments between players
- Cute animations when boys kiss
- Kiss statistics tracking

### Romance Mode Settings
Access through the game mode editor:
- Toggle boys kissing on/off
- Adjust frequency of romantic events (5-50% chance per minute)
- Enable Cupid role for matchmaking gameplay
- Configure heartbreak mechanics

### Chat Commands
Romantic events automatically generate chat messages like:
- "💋 Two boys were spotted sharing a tender kiss under the moonlight! 💕"
- "Alex and Jamie had a cute moment together! 😊💙"
- "Love is in the air! Alex gave Jamie a sweet kiss goodnight! 🌙✨"

## Technical Details

### CSS Styling
Beautiful gradient backgrounds, floating heart animations, and responsive design ensure the feature works well on all devices.

### React Hooks
The `useBoysKissing` hook manages:
- Event generation and timing
- Player name management
- Statistics tracking
- Event history

### Game State Integration
Seamlessly integrates with existing game state without affecting core Mafia mechanics.

## Fun Facts
- Kiss animations include floating hearts, sparkles, and cute emojis
- Players can manually trigger romantic moments
- Statistics track favorite couples and total kisses
- Fully customizable through the romance settings panel
- All text is translatable through the language system

## Future Enhancements
- Seasonal romantic themes (Valentine's Day, etc.)
- More kiss animation varieties
- Voice lines for romantic moments
- Achievement system for romance milestones
- Wedding ceremonies for long-term couples

Enjoy spreading love and joy in your Mafia games! 💖✨
