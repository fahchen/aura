import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useSessionManager } from './hooks/useSessionManager';
import { useDrag } from './hooks/useDrag';
import { Indicator } from './components/Indicator';
import { SessionList } from './components/SessionList';
import { Controls } from './components/Controls';
import { IconPreview } from './IconPreview';

// Initial setup events
const SETUP_EVENTS = [
  // Start 5 sessions to show all states at once
  { type: 'SessionStart', sessionId: 'sess-running', cwd: '/Users/dev/project', name: 'Running State' },
  { type: 'SessionStart', sessionId: 'sess-idle', cwd: '/Users/dev/project', name: 'Idle State' },
  { type: 'SessionStart', sessionId: 'sess-attention', cwd: '/Users/dev/project', name: 'Attention State' },
  { type: 'SessionStart', sessionId: 'sess-compacting', cwd: '/Users/dev/project', name: 'Compacting State' },
  { type: 'SessionStart', sessionId: 'sess-stale', cwd: '/Users/dev/project', name: 'Stale State' },

  // Set each session to its state
  { type: 'PreToolUse', sessionId: 'sess-running', toolId: 't1', toolName: 'Read', toolLabel: 'main.ts' },
  { type: 'Stop', sessionId: 'sess-idle' },
  { type: 'PermissionRequest', sessionId: 'sess-attention' },
  { type: 'PreCompact', sessionId: 'sess-compacting' },
  { type: 'Stale', sessionId: 'sess-stale' },
];

// Random tool names and labels for continuous simulation
const TOOL_NAMES = ['Read', 'Edit', 'Write', 'Bash', 'Grep', 'Glob', 'Task', 'WebFetch', 'mcp__notion__search'];
const TOOL_LABELS = ['main.ts', 'config.json', 'index.tsx', 'npm test', 'TODO', 'src/**/*.ts', 'refactor', 'docs', 'api.ts'];

function getRandomToolEvent(toolId: number): { type: string; sessionId: string; toolId: string; toolName: string; toolLabel: string } {
  return {
    type: 'PreToolUse',
    sessionId: 'sess-running',
    toolId: `t${toolId}`,
    toolName: TOOL_NAMES[Math.floor(Math.random() * TOOL_NAMES.length)],
    toolLabel: TOOL_LABELS[Math.floor(Math.random() * TOOL_LABELS.length)],
  };
}

const SIMULATION_INTERVAL_MS = 800;

export default function App() {
  const { sessions, handleEvent, clearAll } = useSessionManager();
  const [isExpanded, setIsExpanded] = useState(false);
  const [bgClass, setBgClass] = useState('');
  const [bgImage, setBgImage] = useState<string | null>(null);
  const [simulationRunning, setSimulationRunning] = useState(false);
  const [listStyle, setListStyle] = useState<'card' | 'full-width'>('card');
  const [showIconPreview, setShowIconPreview] = useState(false);
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
      // First run through setup events
      if (simulationStep.current < SETUP_EVENTS.length) {
        const event = SETUP_EVENTS[simulationStep.current];
        handleEvent(event);
        simulationStep.current += 1;
        return;
      }

      // Then continuously send random tool events for running session
      const toolId = simulationStep.current;
      // Alternate between PreToolUse and PostToolUse
      if (toolId % 2 === 0) {
        handleEvent(getRandomToolEvent(toolId));
      } else {
        handleEvent({ type: 'PostToolUse', sessionId: 'sess-running', toolId: `t${toolId - 1}` });
      }
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

  if (showIconPreview) {
    return (
      <>
        <IconPreview />
        <button
          onClick={() => setShowIconPreview(false)}
          style={{
            position: 'fixed',
            top: 20,
            right: 20,
            padding: '8px 16px',
            background: '#a78bfa',
            color: '#fff',
            border: 'none',
            borderRadius: 8,
            cursor: 'pointer',
          }}
        >
          Back to Prototype
        </button>
      </>
    );
  }

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

      <button
        onClick={() => setShowIconPreview(true)}
        style={{
          position: 'fixed',
          top: 20,
          right: 20,
          padding: '8px 16px',
          background: 'rgba(255,255,255,0.1)',
          color: '#fff',
          border: '1px solid rgba(255,255,255,0.2)',
          borderRadius: 8,
          cursor: 'pointer',
        }}
      >
        Icon Preview
      </button>

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
