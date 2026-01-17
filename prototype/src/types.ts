export type SessionState = 'running' | 'idle' | 'attention' | 'compacting' | 'stale';

export interface RunningTool {
  toolId: string;
  toolName: string;
  toolLabel?: string;
}

export interface Session {
  sessionId: string;
  cwd: string;
  name?: string;
  state: SessionState;
  runningTools: RunningTool[];
}
