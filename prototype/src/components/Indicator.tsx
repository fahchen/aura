import React, { useState, useEffect, useRef } from 'react';
import type { LucideIcon } from 'lucide-react';
import type { Session } from '../types';
import { INDICATOR_ICONS, CREATIVE_ICONS } from '../constants';

interface IndicatorProps {
  sessions: Session[];
  onClick: () => void;
  onDragStart?: (e: React.MouseEvent) => void;
}

type IndicatorState = 'idle' | 'attention' | 'running';

function getIndicatorState(sessions: Session[]): IndicatorState {
  if (sessions.length === 0) {
    return 'idle';
  }

  const states = sessions.map(s => s.state);
  if (states.includes('attention')) {
    return 'attention';
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

export function Indicator({ sessions, onClick, onDragStart }: IndicatorProps) {
  const indicatorState = getIndicatorState(sessions);
  const [currentIndex, setCurrentIndex] = useState(() => Math.floor(Math.random() * CREATIVE_ICONS.length));
  const [nextIndex, setNextIndex] = useState<number | null>(null);
  const [isTransitioning, setIsTransitioning] = useState(false);
  const transitionTimeoutRef = useRef<NodeJS.Timeout | null>(null);

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

  const indicatorClasses = ['indicator', indicatorState].join(' ');

  // Static icon for idle or attention
  if (!shouldCycle) {
    const Icon = INDICATOR_ICONS[indicatorState];
    return (
      <div
        className={indicatorClasses}
        onClick={onClick}
        onMouseDown={onDragStart}
      >
        <div className="indicator-circle">
          <div className="indicator-gloss" />
          <div className="indicator-icon">
            <Icon size={16} strokeWidth={2} />
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
      className={indicatorClasses}
      onClick={onClick}
      onMouseDown={onDragStart}
    >
      <div className="indicator-circle">
        <div className="indicator-gloss" />
        {!isTransitioning && (
          <div className="indicator-icon" key={`current-${currentIndex}`}>
            <CurrentIcon size={16} strokeWidth={2} />
          </div>
        )}
        {isTransitioning && (
          <>
            <div className="indicator-icon slide-exit" key={`exit-${currentIndex}`}>
              <CurrentIcon size={16} strokeWidth={2} />
            </div>
            {NextIcon && (
              <div className="indicator-icon slide-enter" key={`enter-${nextIndex}`}>
                <NextIcon size={16} strokeWidth={2} />
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
