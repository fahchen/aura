import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useSessionManager } from './hooks/useSessionManager';
import { useDrag } from './hooks/useDrag';
import { Indicator } from './components/Indicator';
import { SessionList } from './components/SessionList';
import { Controls } from './components/Controls';

// Simulation events to demonstrate the HUD
const SIMULATION_EVENTS = [
  { type: 'SessionStart', sessionId: 'sess-1', cwd: '/Users/dev/project-alpha', name: '修复登录问题' },
  { type: 'PreToolUse', sessionId: 'sess-1', toolId: 't1', toolName: 'Read', toolLabel: 'main.ts' },
  { type: 'PostToolUse', sessionId: 'sess-1', toolId: 't1' },
  { type: 'PreToolUse', sessionId: 'sess-1', toolId: 't2', toolName: 'Edit', toolLabel: 'config.json' },
  { type: 'SessionStart', sessionId: 'sess-2', cwd: '/Users/dev/another-project', name: 'This is a very long session name that should truncate' },
  { type: 'PreToolUse', sessionId: 'sess-1', toolId: 't3', toolName: 'Bash', toolLabel: 'npm test' },
  { type: 'PermissionRequest', sessionId: 'sess-1' },
  { type: 'PreToolUse', sessionId: 'sess-2', toolId: 't4', toolName: 'Grep', toolLabel: 'TODO' },
  { type: 'PostToolUse', sessionId: 'sess-1', toolId: 't2' },
  { type: 'PreToolUse', sessionId: 'sess-2', toolId: 't5', toolName: 'Write', toolLabel: 'output.txt' },
  { type: 'PostToolUse', sessionId: 'sess-1', toolId: 't3' },
  { type: 'SessionStart', sessionId: 'sess-3', cwd: '/Users/dev/third-project', name: '添加深色模式支持' },
  { type: 'PreCompact', sessionId: 'sess-1' },
  { type: 'PreToolUse', sessionId: 'sess-3', toolId: 't6', toolName: 'mcp__notion__search', toolLabel: 'docs' },
  { type: 'PostToolUse', sessionId: 'sess-2', toolId: 't4' },
  { type: 'PostToolUse', sessionId: 'sess-2', toolId: 't5' },
  { type: 'Stop', sessionId: 'sess-1' },
  { type: 'PostToolUse', sessionId: 'sess-3', toolId: 't6' },
  { type: 'SessionEnd', sessionId: 'sess-2' },
  { type: 'SessionEnd', sessionId: 'sess-3' },
  { type: 'SessionEnd', sessionId: 'sess-1' },
];

const SIMULATION_INTERVAL_MS = 800;

export default function App() {
  const { sessions, handleEvent, clearAll } = useSessionManager();
  const [isExpanded, setIsExpanded] = useState(false);
  const [bgClass, setBgClass] = useState('');
  const [bgImage, setBgImage] = useState<string | null>(null);
  const [simulationRunning, setSimulationRunning] = useState(false);
  const [listStyle, setListStyle] = useState<'card' | 'full-width'>('card');
  const simulationStep = useRef(0);
  const prevSessionCount = useRef(0);
  const { position, isDragging, handleMouseDown } = useDrag();

  // Auto-expand when sessions first appear, auto-collapse when all gone
  useEffect(() => {
    const hadSessions = prevSessionCount.current > 0;
    const hasSessions = sessions.length > 0;

    // Only auto-expand when going from 0 to >0 sessions
    if (!hadSessions && hasSessions) {
      setIsExpanded(true);
    }
    // Auto-collapse when all sessions are gone
    else if (hadSessions && !hasSessions) {
      setIsExpanded(false);
    }

    prevSessionCount.current = sessions.length;
  }, [sessions.length]);

  // Apply background class and image to body
  useEffect(() => {
    document.body.className = bgClass;
    document.body.style.backgroundImage = bgImage ? `url(${bgImage})` : '';
  }, [bgClass, bgImage]);

  // Simulation effect
  useEffect(() => {
    if (!simulationRunning) {
      return;
    }

    const interval = setInterval(() => {
      if (simulationStep.current >= SIMULATION_EVENTS.length) {
        // Reset simulation
        simulationStep.current = 0;
        setSimulationRunning(false);
        return;
      }

      const event = SIMULATION_EVENTS[simulationStep.current];
      handleEvent(event);
      simulationStep.current += 1;
    }, SIMULATION_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [simulationRunning, handleEvent]);

  const handleToggleView = useCallback(() => {
    setIsExpanded(prev => !prev);
  }, []);

  const handleCollapse = useCallback(() => {
    setIsExpanded(false);
  }, []);

  const handleRunSimulation = useCallback(() => {
    clearAll();
    simulationStep.current = 0;
    setSimulationRunning(true);
  }, [clearAll]);

  const handleStopSimulation = useCallback(() => {
    setSimulationRunning(false);
  }, []);

  const handleAddSession = useCallback(
    (cwd: string) => {
      const sessionId = `manual-${Date.now()}`;
      handleEvent({ type: 'SessionStart', sessionId, cwd });
    },
    [handleEvent]
  );

  const handleSetBackground = useCallback((bg: string) => {
    if (bg === 'random') {
      setBgImage(`https://picsum.photos/1920/1080?random=${Date.now()}`);
      setBgClass('');
    } else {
      setBgImage(null);
      setBgClass(bg);
    }
  }, []);

  const handleToggleStyle = useCallback(() => {
    setListStyle(prev => prev === 'card' ? 'full-width' : 'card');
  }, []);

  const containerStyle = {
    transform: `translateX(-50%) translate(${position.x}px, ${position.y}px)`,
    cursor: isDragging ? 'grabbing' : undefined,
  };

  return (
    <>
      <div className="prototype-container" style={containerStyle}>
        <Indicator sessions={sessions} onClick={handleToggleView} onDragStart={handleMouseDown} />
        {isExpanded && sessions.length > 0 && (
          <SessionList
            sessions={sessions}
            onCollapse={handleCollapse}
            listStyle={listStyle}
            onDragStart={handleMouseDown}
          />
        )}
      </div>

      <Controls
        onToggleView={handleToggleView}
        onRunSimulation={handleRunSimulation}
        onStopSimulation={handleStopSimulation}
        onAddSession={handleAddSession}
        onSetBackground={handleSetBackground}
        onToggleStyle={handleToggleStyle}
        simulationRunning={simulationRunning}
        listStyle={listStyle}
      />
    </>
  );
}
