# Tutorial System

The tutorial system provides an interactive way for new players to learn how to play Midnight Machinations. It's designed to be easy to extend with new tutorials.

## Architecture

The tutorial system is **fully client-side** and consists of several key components:

### Core Components

1. **Tutorial Types** (`tutorialTypes.d.tsx`)
   - Defines TypeScript types for tutorials, steps, and completion conditions
   - Makes the system type-safe and self-documenting

2. **Tutorial Manager** (`tutorialManager.tsx`)
   - Singleton that manages tutorial state and progression
   - Handles step navigation (next/previous)
   - Notifies listeners of state changes
   - Tracks current tutorial and step

3. **Tutorial Definitions** (`tutorials.tsx`)
   - Contains all available tutorials
   - Currently includes "Basic Gameplay" tutorial
   - Easy to add new tutorials by creating new Tutorial objects

### UI Components

1. **Tutorial Menu** (`TutorialMenu.tsx`)
   - Entry point for tutorials
   - Displays all available tutorials as cards
   - Allows users to select and start tutorials

2. **Tutorial Screen** (`TutorialScreen.tsx`)
   - Main screen displayed during tutorials
   - Shows simplified game interface
   - Includes exit button

3. **Tutorial Overlay** (`TutorialOverlay.tsx`)
   - Displays current tutorial step as an overlay
   - Shows progress (step X of Y)
   - Provides navigation buttons (Previous/Next)
   - Can be dismissed with Skip/Finish button

## How to Add a New Tutorial

Adding a new tutorial is straightforward:

1. **Create Tutorial Definition**

In `tutorials.tsx`, create a new Tutorial object:

```typescript
export const myNewTutorial: Tutorial = {
    id: "my-tutorial",
    name: "My Tutorial Name",
    description: "What this tutorial teaches",
    order: 2, // Display order
    initialSetup: {
        playerRole: "villager",
        playerCount: 7,
        startPhase: "briefing",
        startDay: 1
    },
    steps: [
        {
            id: "step-1",
            title: "First Step",
            description: "Instructions for the first step",
            completionCondition: { type: "manual" }
        },
        {
            id: "step-2",
            title: "Second Step",
            description: "Instructions for the second step",
            completionCondition: { type: "manual" }
        },
        // Add more steps...
    ]
};
```

2. **Add to Tutorial List**

Add your tutorial to the `ALL_TUTORIALS` array:

```typescript
export const ALL_TUTORIALS: Tutorial[] = [
    basicGameplayTutorial,
    myNewTutorial
];
```

3. **Done!**

The tutorial will automatically appear in the tutorial menu.

## Tutorial Step Types

### Completion Conditions

Steps can have different completion conditions:

- `{ type: "manual" }` - User clicks Next button (default)
- `{ type: "timer", duration: 5000 }` - Auto-advance after duration (ms)
- `{ type: "click", elementId: "button-id" }` - Complete when element is clicked
- `{ type: "phaseChange", targetPhase: "night" }` - Complete when phase changes
- `{ type: "action", actionType: "vote" }` - Complete when action is performed

### Game State Changes

Steps can modify the simulated game state:

```typescript
gameStateChanges: [
    { type: "setPhase", phase: { type: "night" } },
    { type: "addChatMessage", message: "Example message" },
    { type: "setPlayerCount", count: 10 },
    { type: "assignRole", role: "detective" }
]
```

## Features

### Current Features

- ✅ Client-side tutorial system (no server required)
- ✅ Step-by-step guided instructions
- ✅ Progress tracking (step X of Y)
- ✅ Navigation (Next/Previous/Skip)
- ✅ Overlay UI that doesn't block interaction
- ✅ Easy to extend with new tutorials
- ✅ Responsive design (mobile-friendly)
- ✅ Integrated into main menu

### Future Enhancements

Possible future improvements:

- Interactive game simulation (actual gameplay during tutorial)
- Highlighting specific UI elements
- Branching tutorials (different paths based on choices)
- Save/resume tutorial progress
- Achievement tracking
- Multiple language support via translation system

## Usage

### For Players

1. Launch the game
2. Click "Tutorials" button on the start menu
3. Select a tutorial from the list
4. Follow the on-screen instructions
5. Use Previous/Next buttons to navigate
6. Click "Skip" or "Finish" to exit

### For Developers

The tutorial system is designed to be maintenance-free once tutorials are created. The framework handles all the UI, state management, and navigation automatically.

## Technical Details

- **No server dependency**: Tutorials run entirely on the client
- **Type-safe**: Full TypeScript type definitions
- **Reactive**: Uses React state management for UI updates
- **Singleton pattern**: Tutorial manager ensures consistent state
- **CSS Variables**: Styling respects theme system
- **Accessibility**: Proper ARIA labels and keyboard navigation support

## Files Overview

```
client/src/tutorial/
├── tutorialTypes.d.tsx      # Type definitions
├── tutorialManager.tsx      # State management
├── tutorials.tsx            # Tutorial definitions
├── TutorialMenu.tsx         # Tutorial selection UI
├── TutorialScreen.tsx       # Tutorial game screen
├── TutorialOverlay.tsx      # Step instruction overlay
├── tutorialMenu.css         # Menu styling
├── tutorialScreen.css       # Screen styling
└── tutorialOverlay.css      # Overlay styling
```
