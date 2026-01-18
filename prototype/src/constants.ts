import type { SessionState } from './types';
import {
  Cctv,
  Ghost,
  Cookie,
  BellRing,
  Fan,
  Wind,
  Bot,
  Panda,
  MessageSquareCode,
  WandSparkles,
  Sparkles,
  Flame,
  Zap,
  Brain,
  Spotlight,
  BicepsFlexed,
  Rocket,
  Cpu,
  Puzzle,
  Orbit,
  Terminal,
  FileSearchCorner,
  BookSearch,
  Newspaper,
  FilePenLine,
  FileBracesCorner,
  Binoculars,
  MonitorDown,
  Plug,
  Ticket,
  AudioLines,
  type LucideIcon,
} from 'lucide-react';

// Lucide icons for session states (shown in session row)
export const STATE_ICONS: Record<SessionState, LucideIcon> = {
  running: Cctv,
  idle: MessageSquareCode,  // waiting for user input after Stop
  attention: BellRing,      // needs permission to execute tool (highest priority)
  waiting: Fan,             // waiting for user input (idle_prompt)
  compacting: Cookie,
  stale: Ghost,
};

export const STATE_OPACITY: Record<SessionState, number> = {
  running: 1,
  idle: 0.8,
  attention: 1,
  waiting: 1,
  compacting: 0.9,
  stale: 0.8,
};

// Indicator icons (4 states: idle, attention, waiting, running)
export const INDICATOR_ICONS: Record<'idle' | 'attention' | 'waiting' | 'running', LucideIcon> = {
  idle: Panda,         // No sessions
  attention: BellRing, // At least one session needs permission (highest priority)
  waiting: Fan,        // At least one session waiting for input
  running: Bot,        // Other (cycles through creative icons)
};

// Creative icons for cycling when sessions exist
export const CREATIVE_ICONS: LucideIcon[] = [
  WandSparkles,
  Sparkles,
  Flame,
  Zap,
  Brain,
  Spotlight,
  BicepsFlexed,
  Rocket,
  Cpu,
  Puzzle,
  Orbit,
];

// Tool icons mapping (Lucide icons)
export const TOOL_ICONS: Record<string, LucideIcon> = {
  Task: Bot,
  Bash: Terminal,
  Glob: BookSearch,
  Grep: FileSearchCorner,
  Read: Newspaper,
  Edit: FilePenLine,
  Write: FileBracesCorner,
  WebFetch: MonitorDown,
  WebSearch: Binoculars,
  default: Ticket,
  mcp: Plug,
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
export function getToolIcon(toolName: string): LucideIcon {
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

// Placeholder icons for state-specific event lines
export const PLACEHOLDER_ICONS: Partial<Record<SessionState, LucideIcon>> = {
  waiting: Wind,
};

// Default placeholder icon (AudioLines)
export const DEFAULT_PLACEHOLDER_ICON = AudioLines;
