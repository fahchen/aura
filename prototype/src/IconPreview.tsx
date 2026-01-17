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
  Globe,
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

const styles: Record<string, React.CSSProperties> = {
  container: {
    padding: 40,
    fontFamily: 'system-ui, sans-serif',
    background: '#1a1a2e',
    minHeight: '100vh',
    color: '#fff',
  },
  section: {
    marginBottom: 40,
  },
  title: {
    fontSize: 18,
    fontWeight: 600,
    marginBottom: 16,
    color: '#a78bfa',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
    gap: 12,
  },
  card: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: 8,
    padding: 16,
    background: 'rgba(255,255,255,0.05)',
    borderRadius: 12,
    border: '1px solid rgba(255,255,255,0.1)',
  },
  label: {
    fontSize: 11,
    color: 'rgba(255,255,255,0.7)',
    textAlign: 'center',
  },
  lucideIcon: {
    color: 'rgba(255,255,255,0.9)',
  },
  placeholder: {
    fontFamily: "'Maple Mono NF CN', monospace",
    fontSize: 13,
    color: 'rgba(255,255,255,0.5)',
  },
  toolExample: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    padding: '4px 8px',
    background: 'rgba(255,255,255,0.08)',
    borderRadius: 6,
    fontFamily: "'Maple Mono NF CN', monospace",
    fontSize: 12,
    color: 'rgba(255,255,255,0.9)',
  },
  toolExampleIcon: {
    color: 'rgba(255,255,255,0.7)',
    flexShrink: 0,
  },
  wideCard: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: 10,
    padding: 16,
    background: 'rgba(255,255,255,0.05)',
    borderRadius: 12,
    border: '1px solid rgba(255,255,255,0.1)',
    minWidth: 160,
  },
};

export function IconPreview() {
  return (
    <div style={styles.container}>
      <div style={styles.section}>
        <div style={styles.title}>Current Cycling Icons</div>
        <div style={styles.grid}>
          {currentIcons.map(({ name, Icon }) => (
            <div key={name} style={styles.card}>
              <Icon size={iconSize} />
              <span style={styles.label}>{name}</span>
            </div>
          ))}
        </div>
      </div>

      <div style={styles.section}>
        <div style={styles.title}>Session Row State Icons</div>
        <div style={styles.grid}>
          {stateIcons.map(({ name, Icon }) => (
            <div key={name} style={styles.card}>
              <Icon size={iconSize} />
              <span style={styles.label}>{name}</span>
            </div>
          ))}
        </div>
      </div>

      <div style={styles.section}>
        <div style={styles.title}>Indicator Icons</div>
        <div style={styles.grid}>
          {indicatorIcons.map(({ name, Icon }) => (
            <div key={name} style={styles.card}>
              <Icon size={iconSize} />
              <span style={styles.label}>{name}</span>
            </div>
          ))}
        </div>
      </div>

      <div style={styles.section}>
        <div style={styles.title}>Tool Icons (Lucide)</div>
        <div style={styles.grid}>
          {toolIcons.map(({ name, Icon, example }) => (
            <div key={name} style={styles.wideCard}>
              <Icon size={iconSize} style={styles.lucideIcon} />
              <span style={styles.label}>{name}</span>
              <div style={styles.toolExample}>
                <Icon size={12} style={styles.toolExampleIcon} />
                <span>{example}</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div style={styles.section}>
        <div style={styles.title}>Placeholder Texts</div>
        <div style={styles.grid}>
          {placeholderTexts.map((text) => (
            <div key={text} style={styles.card}>
              <span style={styles.placeholder}>{text}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
