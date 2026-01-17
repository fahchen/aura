import type { SessionState } from './types';
import {
  Play,
  Square,
  RefreshCw,
  Pause,
  Moon,
  BellRing,
  Bot,
  type LucideIcon,
} from 'lucide-react';

// Lucide icons for session states
export const STATE_ICONS: Record<SessionState, LucideIcon> = {
  running: Play,
  idle: Square,
  attention: BellRing,
  compacting: RefreshCw,
  stale: Pause,
};

export const STATE_OPACITY: Record<SessionState, number> = {
  running: 1,
  idle: 0.8,
  attention: 1,
  compacting: 0.9,
  stale: 0.6,
};

// Indicator icons include no-sessions state
export const INDICATOR_ICONS: Record<SessionState | 'no-sessions', LucideIcon> = {
  running: Bot,
  idle: Bot,
  attention: BellRing,
  compacting: RefreshCw,
  stale: Bot,
  'no-sessions': Moon,
};

// Tool icons mapping (still using Nerd Font for tools)
export const TOOL_ICONS: Record<string, string> = {
  Task: '\uf544',      // robot
  Bash: '\ue795',      // terminal
  Glob: '\uf07b',      // folder
  Grep: '\uf002',      // search
  Read: '\uf02d',      // book
  Edit: '\uf044',      // pencil
  Write: '\uf15c',     // file
  WebFetch: '\uf0ac',  // globe
  WebSearch: '\uf002', // search
  default: '\uf013',   // gear
  mcp: '\uf1e6',       // plug
};

// Placeholder texts when no tools are running
export const PLACEHOLDER_TEXTS = [
  'thinking...',
  'drafting...',
  'building...',
  'planning...',
  'analyzing...',
  'pondering...',
  'processing...',
  'reasoning...',
];

/**
 * Get the icon for a tool name
 */
export function getToolIcon(toolName: string): string {
  // Check for MCP tools (start with mcp__)
  if (toolName.startsWith('mcp__')) {
    return TOOL_ICONS.mcp;
  }

  return TOOL_ICONS[toolName] ?? TOOL_ICONS.default;
}

/**
 * Get a random placeholder text
 */
export function getRandomPlaceholder(): string {
  const index = Math.floor(Math.random() * PLACEHOLDER_TEXTS.length);
  return PLACEHOLDER_TEXTS[index];
}
