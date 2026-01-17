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
    </div>
  );
}
