import React, { useState, useEffect, useRef } from 'react';
import type { LucideIcon } from 'lucide-react';
import type { Session, SessionState } from '../types';
import { INDICATOR_ICONS, CREATIVE_ICONS } from '../constants';

interface IndicatorProps {
  sessions: Session[];
  onClick: () => void;
  onDragStart?: (e: React.MouseEvent) => void;
}

type AggregateState = SessionState | 'no-sessions';

function getAggregateState(sessions: Session[]): AggregateState {
  if (sessions.length === 0) {
    return 'no-sessions';
  }

  // Priority: attention > running > compacting > idle > stale
  const states = sessions.map(s => s.state);

  if (states.includes('attention')) return 'attention';
  if (states.includes('running')) return 'running';
  if (states.includes('compacting')) return 'compacting';
  if (states.includes('idle')) return 'idle';
  if (states.includes('stale')) return 'stale';

  return 'no-sessions';
}

const CYCLE_INTERVAL_MS = 2500;
const FADE_DURATION_MS = 300;

export function Indicator({ sessions, onClick, onDragStart }: IndicatorProps) {
  const aggregateState = getAggregateState(sessions);
  const [iconIndex, setIconIndex] = useState(0);
  const [isFading, setIsFading] = useState(false);
  const pendingIndex = useRef<number | null>(null);

  // Cycle through creative icons only when running or stale
  const shouldCycle = sessions.length > 0 && (aggregateState === 'running' || aggregateState === 'stale');

  useEffect(() => {
    if (!shouldCycle) {
      return;
    }

    const interval = setInterval(() => {
      // Start fade out
      setIsFading(true);
      pendingIndex.current = (iconIndex + 1) % CREATIVE_ICONS.length;

      // After fade out, change icon and fade in
      setTimeout(() => {
        setIconIndex(pendingIndex.current!);
        setIsFading(false);
      }, FADE_DURATION_MS);
    }, CYCLE_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [shouldCycle, iconIndex]);

  // Determine which icon to show
  let Icon: LucideIcon;
  if (shouldCycle) {
    Icon = CREATIVE_ICONS[iconIndex];
  } else {
    Icon = INDICATOR_ICONS[aggregateState];
  }

  const indicatorClasses = ['indicator', aggregateState].join(' ');
  const iconClasses = ['indicator-icon', isFading ? 'fading' : ''].filter(Boolean).join(' ');

  return (
    <div
      className={indicatorClasses}
      onClick={onClick}
      onMouseDown={onDragStart}
    >
      <div className="indicator-circle">
        <div className="indicator-gloss" />
        <div className={iconClasses}>
          <Icon size={16} strokeWidth={2} />
        </div>
      </div>
    </div>
  );
}
