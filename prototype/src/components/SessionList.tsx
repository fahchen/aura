import React from 'react';
import type { Session } from '../types';
import { SessionRow } from './SessionRow';

interface SessionListProps {
  sessions: Session[];
  onDragStart?: (e: React.MouseEvent) => void;
  onRemoveSession?: (sessionId: string) => void;
  useShadow?: boolean;
}

export function SessionList({
  sessions,
  onDragStart,
  onRemoveSession,
  useShadow = false,
}: SessionListProps) {
  const sessionCount = sessions.length;

  return (
    <div className="relative w-80 flex flex-col origin-top animate-expand-in">
      {/* Background layer */}
      <div className={`glass-session-list-bg z-[1] ${useShadow ? 'use-shadow' : ''}`} />

      {/* Header content - draggable */}
      <div
        className="relative z-[3] flex items-center justify-center px-3 py-1.5 h-7 cursor-grab active:cursor-grabbing"
        onMouseDown={onDragStart}
      >
        <span className="text-[11px] font-normal theme-text-header">
          {sessionCount} session{sessionCount !== 1 ? 's' : ''}
        </span>
      </div>

      {/* Sessions container */}
      <div className="glass-session-content z-[2]">
        {sessions.map(session => (
          <SessionRow
            key={session.sessionId}
            session={session}
            onRemove={onRemoveSession}
          />
        ))}
      </div>
    </div>
  );
}
