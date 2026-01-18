import React, { useState, useEffect, useRef } from 'react';
import type { Session } from '../types';
import { INDICATOR_ICONS, CREATIVE_ICONS } from '../constants';

interface IndicatorProps {
  sessions: Session[];
  onClick: () => void;
  onDragStart?: (e: React.MouseEvent) => void;
  useShadow?: boolean;
  onToggleTheme?: () => void;
}

type IndicatorState = 'idle' | 'attention' | 'waiting' | 'running';

function getIndicatorState(sessions: Session[]): IndicatorState {
  if (sessions.length === 0) {
    return 'idle';
  }

  const states = sessions.map(s => s.state);
  // Priority: attention > waiting > running
  if (states.includes('attention')) {
    return 'attention';
  }
  if (states.includes('waiting')) {
    return 'waiting';
  }

  return 'running';
}

// Get a random index different from the current one
function getRandomIndex(currentIndex: number): number {
  const len = CREATIVE_ICONS.length;
  if (len <= 1) return 0;
  let nextIndex: number;
  do {
    nextIndex = Math.floor(Math.random() * len);
  } while (nextIndex === currentIndex);
  return nextIndex;
}

const CYCLE_INTERVAL_MS = 2500;
const SLIDE_DURATION_MS = 400;

export function Indicator({ sessions, onClick, onDragStart, useShadow = false, onToggleTheme }: IndicatorProps) {
  const indicatorState = getIndicatorState(sessions);
  const [currentIndex, setCurrentIndex] = useState(() => Math.floor(Math.random() * CREATIVE_ICONS.length));
  const [nextIndex, setNextIndex] = useState<number | null>(null);
  const [isTransitioning, setIsTransitioning] = useState(false);
  const transitionTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const clickTimesRef = useRef<number[]>([]);

  // Triple-click theme switching
  const handleClick = () => {
    const now = Date.now();
    // Filter to keep only clicks within 500ms, then add current click
    clickTimesRef.current = [...clickTimesRef.current.filter(t => now - t < 500), now];

    if (clickTimesRef.current.length >= 3) {
      onToggleTheme?.();
      clickTimesRef.current = [];
      return;
    }
    onClick();
  };

  // Cycle through creative icons only in 'running' state
  const shouldCycle = indicatorState === 'running';

  useEffect(() => {
    if (!shouldCycle) {
      return;
    }

    const interval = setInterval(() => {
      // Start transition: set next icon and trigger animation
      const next = getRandomIndex(currentIndex);
      setNextIndex(next);
      setIsTransitioning(true);

      // After animation completes, update current index
      transitionTimeoutRef.current = setTimeout(() => {
        setCurrentIndex(next);
        setNextIndex(null);
        setIsTransitioning(false);
      }, SLIDE_DURATION_MS);
    }, CYCLE_INTERVAL_MS);

    return () => {
      clearInterval(interval);
      if (transitionTimeoutRef.current) {
        clearTimeout(transitionTimeoutRef.current);
      }
    };
  }, [shouldCycle, currentIndex]);

  // Base classes for the indicator container
  const indicatorClasses = 'cursor-grab active:cursor-grabbing transition-opacity duration-300';

  // Circle classes based on state
  const circleClasses = [
    'glass-indicator',
    useShadow ? 'use-shadow' : '',
    indicatorState === 'attention' ? 'animate-shake animate-pulse-attention' : '',
    'group-hover:glass-indicator-hover',
  ].filter(Boolean).join(' ');

  // Icon color - always use theme indicator icon color
  const iconColorClass = 'theme-icon-indicator';

  // Static icon for idle, permission, or waiting
  if (!shouldCycle) {
    const Icon = INDICATOR_ICONS[indicatorState];
    const spinClass = indicatorState === 'waiting' ? 'animate-spin-slow' : '';
    return (
      <div
        className={`group ${indicatorClasses}`}
        onClick={handleClick}
        onMouseDown={onDragStart}
      >
        <div className={circleClasses}>
          <div className="indicator-gloss" />
          <div className={`absolute inset-0 flex items-center justify-center ${iconColorClass}`}>
            <Icon size={16} strokeWidth={2} className={spinClass} />
          </div>
        </div>
      </div>
    );
  }

  // Cycling icons with slide transition
  const CurrentIcon = CREATIVE_ICONS[currentIndex];
  const NextIcon = nextIndex !== null ? CREATIVE_ICONS[nextIndex] : null;

  return (
    <div
      className={`group ${indicatorClasses}`}
      onClick={handleClick}
      onMouseDown={onDragStart}
    >
      <div className={circleClasses}>
        <div className="indicator-gloss" />
        {!isTransitioning && (
          <div className={`absolute inset-0 flex items-center justify-center ${iconColorClass}`} key={`current-${currentIndex}`}>
            <CurrentIcon size={16} strokeWidth={2} />
          </div>
        )}
        {isTransitioning && (
          <>
            <div className={`absolute inset-0 flex items-center justify-center animate-slide-exit ${iconColorClass}`} key={`exit-${currentIndex}`}>
              <CurrentIcon size={16} strokeWidth={2} />
            </div>
            {NextIcon && (
              <div className={`absolute inset-0 flex items-center justify-center animate-slide-enter ${iconColorClass}`} key={`enter-${nextIndex}`}>
                <NextIcon size={16} strokeWidth={2} />
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
