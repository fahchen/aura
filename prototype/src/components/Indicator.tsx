import React from 'react';
import type { Session, SessionState } from '../types';
import { INDICATOR_ICONS } from '../constants';

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

export function Indicator({ sessions, onClick, onDragStart }: IndicatorProps) {
  const aggregateState = getAggregateState(sessions);
  const Icon = INDICATOR_ICONS[aggregateState];

  const indicatorClasses = ['indicator', aggregateState].join(' ');

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
