import { useState, useRef, useCallback } from 'react';
import type { Session, RunningTool, SessionState } from '../types';

const STALE_TIMEOUT_MS = 60_000; // 60 seconds

interface SessionEvent {
  type: string;
  sessionId: string;
  cwd?: string;
  name?: string;
  toolId?: string;
  toolName?: string;
  toolLabel?: string;
}

export function useSessionManager() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const staleTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const resetStaleTimer = useCallback((sessionId: string) => {
    // Clear existing timer
    const existingTimer = staleTimers.current.get(sessionId);
    if (existingTimer) {
      clearTimeout(existingTimer);
    }

    // Set new timer
    const timer = setTimeout(() => {
      setSessions(prev =>
        prev.map(s =>
          s.sessionId === sessionId ? { ...s, state: 'stale' as SessionState } : s
        )
      );
    }, STALE_TIMEOUT_MS);

    staleTimers.current.set(sessionId, timer);
  }, []);

  const clearStaleTimer = useCallback((sessionId: string) => {
    const timer = staleTimers.current.get(sessionId);
    if (timer) {
      clearTimeout(timer);
      staleTimers.current.delete(sessionId);
    }
  }, []);

  const handleEvent = useCallback((msg: SessionEvent) => {
    const { type, sessionId } = msg;

    switch (type) {
      case 'SessionStart': {
        setSessions(prev => {
          // Check if session already exists
          if (prev.some(s => s.sessionId === sessionId)) {
            return prev;
          }
          return [
            ...prev,
            {
              sessionId,
              cwd: msg.cwd ?? '/',
              name: msg.name,
              state: 'running',
              runningTools: [],
            },
          ];
        });
        resetStaleTimer(sessionId);
        break;
      }

      case 'PreToolUse': {
        const tool: RunningTool = {
          toolId: msg.toolId ?? `tool-${Date.now()}`,
          toolName: msg.toolName ?? 'Unknown',
          toolLabel: msg.toolLabel,
        };
        setSessions(prev =>
          prev.map(s => {
            if (s.sessionId !== sessionId) return s;
            // Avoid duplicate tools
            if (s.runningTools.some(t => t.toolId === tool.toolId)) return s;
            return {
              ...s,
              state: 'running',
              runningTools: [...s.runningTools, tool],
            };
          })
        );
        resetStaleTimer(sessionId);
        break;
      }

      case 'PostToolUse': {
        setSessions(prev =>
          prev.map(s => {
            if (s.sessionId !== sessionId) return s;
            return {
              ...s,
              runningTools: s.runningTools.filter(t => t.toolId !== msg.toolId),
            };
          })
        );
        resetStaleTimer(sessionId);
        break;
      }

      case 'PermissionRequest': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId ? { ...s, state: 'attention' } : s
          )
        );
        resetStaleTimer(sessionId);
        break;
      }

      case 'Stop': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId
              ? { ...s, state: 'idle', runningTools: [] }
              : s
          )
        );
        resetStaleTimer(sessionId);
        break;
      }

      case 'PreCompact': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId ? { ...s, state: 'compacting' } : s
          )
        );
        resetStaleTimer(sessionId);
        break;
      }

      case 'SessionEnd': {
        clearStaleTimer(sessionId);
        setSessions(prev => prev.filter(s => s.sessionId !== sessionId));
        break;
      }

      // Health check events - set to running
      case 'Notification':
      case 'SubagentStop':
      case 'UserPromptSubmit': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId ? { ...s, state: 'running' } : s
          )
        );
        resetStaleTimer(sessionId);
        break;
      }

      default:
        // Unknown event type, just reset stale timer if session exists
        resetStaleTimer(sessionId);
        break;
    }
  }, [resetStaleTimer, clearStaleTimer]);

  const clearAll = useCallback(() => {
    // Clear all stale timers
    staleTimers.current.forEach(timer => clearTimeout(timer));
    staleTimers.current.clear();
    setSessions([]);
  }, []);

  return { sessions, handleEvent, clearAll };
}
