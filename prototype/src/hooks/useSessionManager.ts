import { useState, useCallback } from 'react';
import type { Session, RunningTool } from '../types';

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

  const handleEvent = useCallback((msg: SessionEvent) => {
    const { type, sessionId } = msg;

    switch (type) {
      case 'SessionStart': {
        setSessions(prev => {
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
            if (s.runningTools.some(t => t.toolId === tool.toolId)) return s;
            return {
              ...s,
              state: 'running',
              runningTools: [...s.runningTools, tool],
            };
          })
        );
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
        break;
      }

      case 'PermissionRequest': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId
              ? { ...s, state: 'attention', permissionTool: msg.toolName }
              : s
          )
        );
        break;
      }

      case 'Stop': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId
              ? { ...s, state: 'idle', runningTools: [], stoppedAt: Date.now() }
              : s
          )
        );
        break;
      }

      case 'PreCompact': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId ? { ...s, state: 'compacting' } : s
          )
        );
        break;
      }

      case 'SessionEnd': {
        setSessions(prev => prev.filter(s => s.sessionId !== sessionId));
        break;
      }

      case 'Notification':
      case 'SubagentStop':
      case 'UserPromptSubmit': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId ? { ...s, state: 'running' } : s
          )
        );
        break;
      }

      case 'Stale': {
        setSessions(prev =>
          prev.map(s =>
            s.sessionId === sessionId ? { ...s, state: 'stale', staleAt: Date.now() } : s
          )
        );
        break;
      }
    }
  }, []);

  const clearAll = useCallback(() => {
    setSessions([]);
  }, []);

  const removeSession = useCallback((sessionId: string) => {
    setSessions(prev => prev.filter(s => s.sessionId !== sessionId));
  }, []);

  return { sessions, handleEvent, clearAll, removeSession };
}
