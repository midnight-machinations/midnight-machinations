import React, { useState } from 'react';
import translate from '../../game/lang';
import { Button } from '../Button';
import CheckBox from '../CheckBox';
import './romanceModeSettings.css';

interface RomanceModeSettingsProps {
    onSettingsChange?: (settings: RomanceModeSettings) => void;
}

export interface RomanceModeSettings {
    enableBoysKissing: boolean;
    romanticEventFrequency: number;
    cupidModeEnabled: boolean;
    lovebirdsEnabled: boolean;
    heartBreakKillsEnabled: boolean;
    randomRomanticMoments: boolean;
}

const DEFAULT_ROMANCE_SETTINGS: RomanceModeSettings = {
    enableBoysKissing: true,
    romanticEventFrequency: 0.15,
    cupidModeEnabled: false,
    lovebirdsEnabled: false,
    heartBreakKillsEnabled: false,
    randomRomanticMoments: true
};

export default function RomanceModeSettings({ onSettingsChange }: RomanceModeSettingsProps) {
    const [settings, setSettings] = useState<RomanceModeSettings>(DEFAULT_ROMANCE_SETTINGS);

    const updateSetting = <K extends keyof RomanceModeSettings>(
        key: K, 
        value: RomanceModeSettings[K]
    ) => {
        const newSettings = { ...settings, [key]: value };
        setSettings(newSettings);
        if (onSettingsChange) {
            onSettingsChange(newSettings);
        }
    };

    const presetRomanticMode = () => {
        const romanticSettings: RomanceModeSettings = {
            enableBoysKissing: true,
            romanticEventFrequency: 0.25,
            cupidModeEnabled: true,
            lovebirdsEnabled: true,
            heartBreakKillsEnabled: true,
            randomRomanticMoments: true
        };
        setSettings(romanticSettings);
        if (onSettingsChange) {
            onSettingsChange(romanticSettings);
        }
    };

    return (
        <div className="romance-mode-settings">
            <div className="settings-header">
                <h3>💕 Romance Mode Settings 💕</h3>
                <p>Configure how love and boys kissing work in your game!</p>
            </div>

            <div className="settings-grid">
                <div className="setting-item">
                    <CheckBox
                        checked={settings.enableBoysKissing}
                        onChange={(value) => updateSetting('enableBoysKissing', value)}
                    />
                    <div className="setting-info">
                        <label>💋 Enable Boys Kissing</label>
                        <p>Allow adorable boys kissing moments during the game</p>
                    </div>
                </div>

                <div className="setting-item">
                    <CheckBox
                        checked={settings.randomRomanticMoments}
                        onChange={(value) => updateSetting('randomRomanticMoments', value)}
                    />
                    <div className="setting-info">
                        <label>✨ Random Romantic Moments</label>
                        <p>Spontaneous cute moments between players</p>
                    </div>
                </div>

                <div className="setting-item">
                    <label htmlFor="frequency-slider">💝 Romance Frequency</label>
                    <input
                        id="frequency-slider"
                        type="range"
                        min="0.05"
                        max="0.5"
                        step="0.05"
                        value={settings.romanticEventFrequency}
                        onChange={(e) => updateSetting('romanticEventFrequency', parseFloat(e.target.value))}
                        disabled={!settings.enableBoysKissing || !settings.randomRomanticMoments}
                        className="frequency-slider"
                    />
                    <span className="frequency-value">
                        {Math.round(settings.romanticEventFrequency * 100)}% chance per minute
                    </span>
                </div>

                <div className="setting-item">
                    <CheckBox
                        checked={settings.cupidModeEnabled}
                        onChange={(value) => updateSetting('cupidModeEnabled', value)}
                    />
                    <div className="setting-info">
                        <label>🏹 Cupid Role Enabled</label>
                        <p>Add the Cupid role that can make players fall in love</p>
                    </div>
                </div>

                <div className="setting-item">
                    <CheckBox
                        checked={settings.lovebirdsEnabled}
                        onChange={(value) => updateSetting('lovebirdsEnabled', value)}
                    />
                    <div className="setting-info">
                        <label>🐦 Lovebird Mechanic</label>
                        <p>Matched players become lovebirds and share a win condition</p>
                    </div>
                </div>

                <div className="setting-item">
                    <CheckBox
                        checked={settings.heartBreakKillsEnabled}
                        onChange={(value) => updateSetting('heartBreakKillsEnabled', value)}
                    />
                    <div className="setting-info">
                        <label>💔 Heartbreak Deaths</label>
                        <p>If one lovebird dies, the other dies of heartbreak</p>
                    </div>
                </div>
            </div>

            <div className="preset-buttons">
                <Button 
                    onClick={presetRomanticMode}
                    className="preset-romantic"
                >
                    💖 Full Romance Mode
                </Button>
                
                <Button 
                    onClick={() => {
                        setSettings(DEFAULT_ROMANCE_SETTINGS);
                        if (onSettingsChange) onSettingsChange(DEFAULT_ROMANCE_SETTINGS);
                    }}
                    className="preset-default"
                >
                    🔄 Reset to Default
                </Button>
            </div>

            <div className="romance-tips">
                <h4>💡 Romance Mode Tips:</h4>
                <ul>
                    <li>Boys kissing events boost town morale and reduce suspicion</li>
                    <li>Cupid can create powerful alliances through love</li>
                    <li>Heartbreak mechanics add emotional stakes to the game</li>
                    <li>Romance events are purely for fun and don't affect gameplay mechanics unless roles are enabled</li>
                </ul>
            </div>
        </div>
    );
}

export { DEFAULT_ROMANCE_SETTINGS };
