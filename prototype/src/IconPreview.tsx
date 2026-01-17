import React from 'react';
import {
  // Current cycling icons
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
  // State icons
  Cctv,
  Ghost,
  Cookie,
  BellRing,
  MessageSquareCode,
  Panda,
  // Tool icons
  Bot,
  Terminal,
  BookSearch,
  FileSearchCorner,
  Newspaper,
  FilePenLine,
  FileBracesCorner,
  Binoculars,
  MonitorDown,
  Plug,
  Ticket,
  type LucideIcon,
} from 'lucide-react';

const iconSize = 24;

const currentIcons = [
  { name: 'WandSparkles', Icon: WandSparkles },
  { name: 'Sparkles', Icon: Sparkles },
  { name: 'Flame', Icon: Flame },
  { name: 'Zap', Icon: Zap },
  { name: 'Brain', Icon: Brain },
  { name: 'Spotlight', Icon: Spotlight },
  { name: 'BicepsFlexed', Icon: BicepsFlexed },
  { name: 'Rocket', Icon: Rocket },
  { name: 'Cpu', Icon: Cpu },
  { name: 'Puzzle', Icon: Puzzle },
  { name: 'Orbit', Icon: Orbit },
];

const stateIcons = [
  { name: 'running: Cctv', Icon: Cctv },
  { name: 'idle: MessageSquareCode', Icon: MessageSquareCode },
  { name: 'attention: BellRing', Icon: BellRing },
  { name: 'compacting: Cookie', Icon: Cookie },
  { name: 'stale: Ghost', Icon: Ghost },
];

const indicatorIcons = [
  { name: 'idle: Panda (no sessions)', Icon: Panda },
  { name: 'attention: BellRing (static)', Icon: BellRing },
  { name: 'running: (cycles creative icons)', Icon: WandSparkles },
];

const placeholderTexts = [
  'thinking...',
  'drafting...',
  'building...',
  'planning...',
  'analyzing...',
  'pondering...',
  'processing...',
  'reasoning...',
];

const toolIcons: { name: string; Icon: LucideIcon; example: string }[] = [
  { name: 'Task', Icon: Bot, example: 'refactor auth' },
  { name: 'Bash', Icon: Terminal, example: 'npm test' },
  { name: 'Glob', Icon: BookSearch, example: 'src/**/*.ts' },
  { name: 'Grep', Icon: FileSearchCorner, example: 'TODO' },
  { name: 'Read', Icon: Newspaper, example: 'main.ts' },
  { name: 'Edit', Icon: FilePenLine, example: 'config.json' },
  { name: 'Write', Icon: FileBracesCorner, example: 'index.tsx' },
  { name: 'WebFetch', Icon: MonitorDown, example: 'docs' },
  { name: 'WebSearch', Icon: Binoculars, example: 'api reference' },
  { name: 'mcp__*', Icon: Plug, example: 'notion search' },
  { name: '(default)', Icon: Ticket, example: 'custom tool' },
];

export function IconPreview() {
  return (
    <div className="p-10 font-sans bg-[#1a1a2e] min-h-screen text-white">
      <div className="mb-10">
        <div className="text-lg font-semibold mb-4 text-purple-400">Current Cycling Icons</div>
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-3">
          {currentIcons.map(({ name, Icon }) => (
            <div
              key={name}
              className="flex flex-col items-center gap-2 p-4 bg-white/5 rounded-xl border border-white/10"
            >
              <Icon size={iconSize} />
              <span className="text-[11px] text-white/70 text-center">{name}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="mb-10">
        <div className="text-lg font-semibold mb-4 text-purple-400">Session Row State Icons</div>
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-3">
          {stateIcons.map(({ name, Icon }) => (
            <div
              key={name}
              className="flex flex-col items-center gap-2 p-4 bg-white/5 rounded-xl border border-white/10"
            >
              <Icon size={iconSize} />
              <span className="text-[11px] text-white/70 text-center">{name}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="mb-10">
        <div className="text-lg font-semibold mb-4 text-purple-400">Indicator Icons</div>
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-3">
          {indicatorIcons.map(({ name, Icon }) => (
            <div
              key={name}
              className="flex flex-col items-center gap-2 p-4 bg-white/5 rounded-xl border border-white/10"
            >
              <Icon size={iconSize} />
              <span className="text-[11px] text-white/70 text-center">{name}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="mb-10">
        <div className="text-lg font-semibold mb-4 text-purple-400">Tool Icons (Lucide)</div>
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-3">
          {toolIcons.map(({ name, Icon, example }) => (
            <div
              key={name}
              className="flex flex-col items-center gap-2.5 p-4 bg-white/5 rounded-xl border border-white/10 min-w-[160px]"
            >
              <Icon size={iconSize} className="text-white/90" />
              <span className="text-[11px] text-white/70 text-center">{name}</span>
              <div className="flex items-center gap-1.5 px-2 py-1 bg-white/[0.08] rounded-md font-mono text-xs text-white/90">
                <Icon size={12} className="text-white/70 shrink-0" />
                <span>{example}</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="mb-10">
        <div className="text-lg font-semibold mb-4 text-purple-400">Placeholder Texts</div>
        <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-3">
          {placeholderTexts.map((text) => (
            <div
              key={text}
              className="flex flex-col items-center gap-2 p-4 bg-white/5 rounded-xl border border-white/10"
            >
              <span className="font-mono text-[13px] text-white/50">{text}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
