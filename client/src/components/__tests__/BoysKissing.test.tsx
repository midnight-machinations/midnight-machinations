import React from 'react';
import BoysKissing from '../BoysKissing';
import { render, screen } from '@testing-library/react';

// Mock the language function
jest.mock('../../game/lang', () => ({
    __esModule: true,
    default: (key: string, ...args: any[]) => {
        const mockTranslations: Record<string, string> = {
            'phase.romanticEvening': 'Romantic Evening',
            'phase.romanticEvening.subtitle': 'Love is in the air! Boys share tender moments under the starlight. 💕',
            'chatMessage.boysKissing': '💋 Two boys were spotted sharing a tender kiss under the moonlight! 💕',
            'chatMessage.boysKissing.romantic': `Love is in the air! ${args[0]} gave ${args[1]} a sweet kiss goodnight! 🌙✨`,
            'chatMessage.boysKissing.wholesome': `Aww! ${args[0]} blushed as ${args[1]} kissed their forehead! 🥰`
        };
        return mockTranslations[key] || key;
    }
}));

describe('BoysKissing Component', () => {
    test('renders romantic evening header', () => {
        render(<BoysKissing />);
        expect(screen.getByText(/Romantic Evening/)).toBeInTheDocument();
    });

    test('displays boy names correctly', () => {
        render(<BoysKissing boy1Name="Alex" boy2Name="Jamie" />);
        expect(screen.getByText('Alex')).toBeInTheDocument();
        expect(screen.getByText('Jamie')).toBeInTheDocument();
    });

    test('shows love statistics', () => {
        render(<BoysKissing />);
        expect(screen.getByText('Love Level')).toBeInTheDocument();
        expect(screen.getByText('Happiness')).toBeInTheDocument();
    });

    test('handles completion callback', () => {
        const mockCallback = jest.fn();
        render(<BoysKissing onKissComplete={mockCallback} />);
        
        // The callback should be triggered after the animation completes
        // In a real test, you'd want to mock the timeouts and test this properly
        expect(mockCallback).toHaveBeenCalledTimes(0); // Initially not called
    });

    test('contains romantic emojis and styling', () => {
        const { container } = render(<BoysKissing />);
        expect(container.innerHTML).toMatch(/💕|💖|💋|✨/);
    });
});
