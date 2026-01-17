import React from 'react';
import type { Session } from '../types';
import { SessionRow } from './SessionRow';

interface SessionListProps {
  sessions: Session[];
  onCollapse: () => void;
  listStyle: 'card' | 'full-width';
}

const MAX_VISIBLE_SESSIONS = 5;

export function SessionList({ sessions, onCollapse, listStyle }: SessionListProps) {
  const visibleSessions = sessions.slice(0, MAX_VISIBLE_SESSIONS);
  const sessionCount = sessions.length;

  return (
    <div className={`session-list compact ${listStyle}`}>
      {/* Background layer */}
      <div className="session-list-header" />

      {/* Header content - compact style */}
      <div className="session-list-header-content">
        <div className="session-list-title">
          <span className="session-list-title-icon">{'\uf489'}</span>
          <span className="session-list-title-count">
            {sessionCount} session{sessionCount !== 1 ? 's' : ''}
          </span>
        </div>
        <button className="session-list-close" onClick={onCollapse}>
          ✕
        </button>
      </div>

      {/* Sessions container */}
      <div className="session-list-content">
        {visibleSessions.map(session => (
          <SessionRow key={session.sessionId} session={session} />
        ))}
      </div>
    </div>
  );
}
