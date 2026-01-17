import React, { memo, useState, useEffect, useRef } from 'react';
import type { Session } from '../types';
import { STATE_ICONS, STATE_OPACITY, getToolIcon, getRandomPlaceholder } from '../constants';

interface SessionRowProps {
  session: Session;
}

function SessionRowInner({ session }: SessionRowProps) {
  const { sessionId, cwd, name, state, runningTools } = session;
  const [toolIndex, setToolIndex] = useState(0);
  const isFirstMount = useRef(true);
  const placeholderRef = useRef(getRandomPlaceholder());

  // Tool cycling effect - only when multiple tools
  useEffect(() => {
    if (runningTools.length <= 1) {
      setToolIndex(0);
      return;
    }

    const interval = setInterval(() => {
      setToolIndex(prev => (prev + 1) % runningTools.length);
    }, 2000);

    return () => clearInterval(interval);
  }, [runningTools.length]);

  // Reset tool index when tools change
  useEffect(() => {
    if (toolIndex >= runningTools.length) {
      setToolIndex(0);
    }
  }, [runningTools.length, toolIndex]);

  // Mark first mount complete after initial render
  useEffect(() => {
    if (isFirstMount.current) {
      // Use requestAnimationFrame to ensure animation plays, then mark as mounted
      requestAnimationFrame(() => {
        isFirstMount.current = false;
      });
    }
  }, []);

  // Extract display name
  const displayName = name ?? cwd.split('/').filter(Boolean).pop() ?? 'Unknown';

  // Get current tool
  const currentTool = runningTools[toolIndex];

  // Build class names
  const rowClasses = [
    'session-row',
    state,
    isFirstMount.current ? 'slideIn' : '',
  ].filter(Boolean).join(' ');

  const stateIndicatorClasses = [
    'state-indicator',
    state === 'attention' ? 'attention' : '',
    state === 'compacting' ? 'compacting' : '',
  ].filter(Boolean).join(' ');

  const StateIcon = STATE_ICONS[state];

  return (
    <div className={rowClasses} data-session-id={sessionId}>
      <div className="session-header">
        <div className={stateIndicatorClasses} style={{ opacity: STATE_OPACITY[state] }}>
          <StateIcon size={14} strokeWidth={2} />
        </div>
        <div className="session-name">
          <span className="session-name-text">{displayName}</span>
        </div>
      </div>
      <div className="session-event">
        {currentTool ? (
          <div className="tool-item">
            <span className="tool-icon">{getToolIcon(currentTool.toolName)}</span>
            <span className="tool-label">
              {currentTool.toolLabel ?? currentTool.toolName}
            </span>
          </div>
        ) : (
          <span className="tool-placeholder">{placeholderRef.current}</span>
        )}
      </div>
    </div>
  );
}

// Memoize to prevent unnecessary re-renders
export const SessionRow = memo(SessionRowInner);
