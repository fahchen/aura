import React, { memo, useState, useEffect, useRef, useCallback } from 'react';
import type { Session } from '../types';
import { STATE_ICONS, STATE_OPACITY, getToolIcon, getRandomPlaceholder } from '../constants';
import { Bomb } from 'lucide-react';

interface SessionRowProps {
  session: Session;
  onRemove?: (sessionId: string) => void;
}

function formatDateTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleString([], {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function SessionRowInner({ session, onRemove }: SessionRowProps) {
  const { sessionId, cwd, name, state, runningTools, stoppedAt, staleAt, permissionTool } = session;
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
      requestAnimationFrame(() => {
        isFirstMount.current = false;
      });
    }
  }, []);

  const handleRemove = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    if (onRemove) {
      onRemove(sessionId);
    }
  }, [onRemove, sessionId]);

  // Extract display name
  const displayName = name ?? cwd.split('/').filter(Boolean).pop() ?? 'Unknown';

  // Get current tool
  const currentTool = runningTools[toolIndex];

  // Build row classes
  const rowClasses = [
    'group glass-session-row',
    isFirstMount.current ? 'animate-slide-in' : '',
    state === 'idle' ? 'opacity-70' : '',
    state === 'stale' ? 'opacity-50 animate-breathe' : '',
    state === 'attention' ? 'shadow-attention' : '',
  ].filter(Boolean).join(' ');

  // State indicator classes
  const stateIndicatorClasses = [
    'shrink-0 w-4 h-4 flex items-center justify-center',
    'font-mono text-sm text-white/70 text-shadow-sm',
    state === 'attention' ? 'animate-shake' : '',
    onRemove ? 'cursor-pointer relative overflow-hidden' : '',
  ].filter(Boolean).join(' ');

  const StateIcon = STATE_ICONS[state];
  const opacity = STATE_OPACITY[state];

  return (
    <div className={rowClasses} data-session-id={sessionId}>
      <div className="flex flex-row items-center gap-2">
        <div
          className={stateIndicatorClasses}
          style={{ opacity }}
          onClick={onRemove ? handleRemove : undefined}
        >
          {onRemove ? (
            <>
              {/* Default icon - slides right on hover */}
              <span className="state-icon-slide group-hover:translate-x-4 group-hover:opacity-0">
                <StateIcon size={14} strokeWidth={2} />
              </span>
              {/* Remove icon - slides in from left on hover */}
              <span className="state-icon-slide absolute inset-0 -translate-x-4 opacity-0 group-hover:translate-x-0 group-hover:opacity-100">
                <Bomb size={14} strokeWidth={2} />
              </span>
            </>
          ) : (
            <StateIcon size={14} strokeWidth={2} />
          )}
        </div>
        <div className="flex-1 min-w-0 overflow-hidden">
          <span className="font-mono text-sm text-white/95 font-medium whitespace-nowrap overflow-hidden text-ellipsis text-shadow-md block">
            {displayName}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-1.5 pl-6 min-h-[18px]">
        {state === 'idle' && stoppedAt ? (
          <span className="text-xs text-white/30 italic">waiting since {formatDateTime(stoppedAt)}</span>
        ) : state === 'stale' && staleAt ? (
          <span className="text-xs text-white/30 italic">inactive since {formatDateTime(staleAt)}</span>
        ) : state === 'attention' ? (
          <span className="text-xs text-white/30 italic">{permissionTool ?? 'Tool'} needs permission</span>
        ) : state === 'compacting' ? (
          <span className="text-xs text-white/30 italic">compacting context...</span>
        ) : currentTool ? (
          <div className="flex items-center gap-2 font-mono text-xs text-white/60 text-shadow-sm animate-fade-in-glass">
            {(() => {
              const ToolIcon = getToolIcon(currentTool.toolName);
              return <ToolIcon size={12} strokeWidth={2} className="shrink-0 text-white/50" />;
            })()}
            <span className="whitespace-nowrap overflow-hidden text-ellipsis italic">
              {currentTool.toolLabel ?? currentTool.toolName}
            </span>
          </div>
        ) : (
          <span className="text-xs text-white/30 italic">{placeholderRef.current}</span>
        )}
      </div>
    </div>
  );
}

// Memoize to prevent unnecessary re-renders
export const SessionRow = memo(SessionRowInner, (prevProps, nextProps) => {
  return (
    prevProps.session === nextProps.session &&
    prevProps.onRemove === nextProps.onRemove
  );
});
