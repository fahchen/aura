import React from 'react';
import { Layers } from 'lucide-react';
import type { Session } from '../types';
import { SessionRow } from './SessionRow';

interface SessionListProps {
  sessions: Session[];
  onDragStart?: (e: React.MouseEvent) => void;
  onRemoveSession?: (sessionId: string) => void;
}

export function SessionList({
  sessions,
  onDragStart,
  onRemoveSession,
}: SessionListProps) {
  const sessionCount = sessions.length;

  return (
    <div className="session-list">
      {/* Background layer */}
      <div className="session-list-header" />

      {/* Header content - draggable */}
      <div className="session-list-header-content" onMouseDown={onDragStart}>
        <div className="session-list-title">
          <span className="session-list-title-icon"><Layers size={12} /></span>
          <span className="session-list-title-count">
            {sessionCount} session{sessionCount !== 1 ? 's' : ''}
          </span>
        </div>
      </div>

      {/* Sessions container */}
      <div className="session-list-content">
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
