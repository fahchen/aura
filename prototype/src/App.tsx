import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useSessionManager } from './hooks/useSessionManager';
import { Indicator } from './components/Indicator';
import { SessionList } from './components/SessionList';
import { Controls } from './components/Controls';

// Simulation events to demonstrate the HUD
const SIMULATION_EVENTS = [
  { type: 'SessionStart', sessionId: 'sess-1', cwd: '/Users/dev/project-alpha' },
  { type: 'PreToolUse', sessionId: 'sess-1', toolId: 't1', toolName: 'Read', toolLabel: 'main.ts' },
  { type: 'PostToolUse', sessionId: 'sess-1', toolId: 't1' },
  { type: 'PreToolUse', sessionId: 'sess-1', toolId: 't2', toolName: 'Edit', toolLabel: 'config.json' },
  { type: 'SessionStart', sessionId: 'sess-2', cwd: '/Users/dev/another-project' },
  { type: 'PreToolUse', sessionId: 'sess-1', toolId: 't3', toolName: 'Bash', toolLabel: 'npm test' },
  { type: 'PermissionRequest', sessionId: 'sess-1' },
  { type: 'PreToolUse', sessionId: 'sess-2', toolId: 't4', toolName: 'Grep', toolLabel: 'TODO' },
  { type: 'PostToolUse', sessionId: 'sess-1', toolId: 't2' },
  { type: 'PreToolUse', sessionId: 'sess-2', toolId: 't5', toolName: 'Write', toolLabel: 'output.txt' },
  { type: 'PostToolUse', sessionId: 'sess-1', toolId: 't3' },
  { type: 'SessionStart', sessionId: 'sess-3', cwd: '/Users/dev/third-project', name: 'Custom Name' },
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
  const simulationStep = useRef(0);

  // Auto-expand when sessions appear, auto-collapse when empty
  useEffect(() => {
    if (sessions.length > 0 && !isExpanded) {
      setIsExpanded(true);
    } else if (sessions.length === 0 && isExpanded) {
      setIsExpanded(false);
    }
  }, [sessions.length, isExpanded]);

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

  const handleExpand = useCallback(() => {
    setIsExpanded(true);
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

  return (
    <>
      <div className="prototype-container">
        {isExpanded ? (
          <SessionList sessions={sessions} onCollapse={handleCollapse} />
        ) : (
          <Indicator sessions={sessions} onClick={handleExpand} />
        )}
      </div>

      <Controls
        onToggleView={handleToggleView}
        onRunSimulation={handleRunSimulation}
        onStopSimulation={handleStopSimulation}
        onAddSession={handleAddSession}
        onSetBackground={handleSetBackground}
        simulationRunning={simulationRunning}
      />
    </>
  );
}
