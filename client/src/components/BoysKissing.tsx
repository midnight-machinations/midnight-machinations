import React, { useState, useEffect, useCallback } from 'react';
import translate from '../game/lang';
import './boysKissing.css';

interface BoysKissingProps {
    boy1Name?: string;
    boy2Name?: string;
    onKissComplete?: () => void;
}

export default function BoysKissing({ boy1Name = "Alex", boy2Name = "Jamie", onKissComplete }: BoysKissingProps) {
    const [hearts, setHearts] = useState<{ id: number; x: number; y: number }[]>([]);
    const [kissingState, setKissingState] = useState<'waiting' | 'approaching' | 'kissing' | 'complete'>('waiting');

    const startKissAnimation = useCallback(() => {
        setKissingState('approaching');
        
        // Generate floating hearts
        const newHearts = Array.from({ length: 8 }, (_, i) => ({
            id: Date.now() + i,
            x: Math.random() * 300,
            y: Math.random() * 200
        }));
        setHearts(newHearts);

        // Sequence the animation
        setTimeout(() => setKissingState('kissing'), 1000);
        setTimeout(() => {
            setKissingState('complete');
            if (onKissComplete) onKissComplete();
        }, 2500);
    }, [onKissComplete]);

    useEffect(() => {
        // Auto-start animation after a short delay
        const timer = setTimeout(startKissAnimation, 500);
        return () => clearTimeout(timer);
    }, [startKissAnimation]);

    return (
        <div className="boys-kissing-container">
            <div className="boys-kissing-header">
                <h2>💕 {translate("phase.romanticEvening")} 💕</h2>
                <p>{translate("phase.romanticEvening.subtitle")}</p>
            </div>
            
            <div className="kissing-scene">
                <div className={`boy boy-1 ${kissingState}`}>
                    <div className="boy-emoji">👨</div>
                    <div className="boy-name">{boy1Name}</div>
                </div>

                <div className="kiss-effects">
                    {hearts.map(heart => (
                        <div 
                            key={heart.id} 
                            className="floating-heart"
                            style={{ 
                                left: `${heart.x}px`, 
                                top: `${heart.y}px`,
                                animationDelay: `${Math.random() * 2}s`
                            }}
                        >
                            💖
                        </div>
                    ))}
                    
                    {kissingState === 'kissing' && (
                        <div className="kiss-sparkles">
                            <div className="sparkle sparkle-1">✨</div>
                            <div className="sparkle sparkle-2">💫</div>
                            <div className="sparkle sparkle-3">⭐</div>
                            <div className="kiss-heart">💋</div>
                        </div>
                    )}
                </div>

                <div className={`boy boy-2 ${kissingState}`}>
                    <div className="boy-emoji">👨</div>
                    <div className="boy-name">{boy2Name}</div>
                </div>
            </div>

            <div className="romantic-messages">
                {kissingState === 'waiting' && (
                    <p className="message waiting">{translate("chatMessage.boysKissing")}</p>
                )}
                {kissingState === 'approaching' && (
                    <p className="message approaching">Hearts are racing... 💓</p>
                )}
                {kissingState === 'kissing' && (
                    <p className="message kissing">{translate("chatMessage.boysKissing.romantic", boy1Name, boy2Name)}</p>
                )}
                {kissingState === 'complete' && (
                    <p className="message complete">{translate("chatMessage.boysKissing.wholesome", boy1Name, boy2Name)}</p>
                )}
            </div>

            <div className="love-stats">
                <div className="stat">
                    <span className="stat-icon">💕</span>
                    <span className="stat-label">Love Level</span>
                    <div className="love-meter">
                        <div className={`love-fill ${kissingState}`}></div>
                    </div>
                </div>
                <div className="stat">
                    <span className="stat-icon">😊</span>
                    <span className="stat-label">Happiness</span>
                    <span className="stat-value">{kissingState === 'complete' ? '100%' : '85%'}</span>
                </div>
            </div>
        </div>
    );
}
