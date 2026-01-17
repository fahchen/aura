import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useSessionManager } from './hooks/useSessionManager';
import { useDrag } from './hooks/useDrag';
import { Indicator } from './components/Indicator';
import { SessionList } from './components/SessionList';
import { Controls } from './components/Controls';
import { IconPreview } from './IconPreview';

// Initial setup events
const SETUP_EVENTS = [
  // Start sessions with English, Chinese, Japanese mix (including long names)
  { type: 'SessionStart', sessionId: 'sess-running', cwd: '/Users/dev/project', name: 'Fix Login' },
  { type: 'SessionStart', sessionId: 'sess-idle', cwd: '/Users/dev/project', name: 'Implement User Authentication Flow' },
  { type: 'SessionStart', sessionId: 'sess-attention', cwd: '/Users/dev/project', name: 'バグ修正と機能追加' },
  { type: 'SessionStart', sessionId: 'sess-compacting', cwd: '/Users/dev/project', name: '重构用户认证模块并优化性能' },
  { type: 'SessionStart', sessionId: 'sess-stale', cwd: '/Users/dev/project', name: 'API追加' },
  { type: 'SessionStart', sessionId: 'sess-long', cwd: '/Users/dev/project', name: 'Refactor Database Connection Pooling and Implement Retry Logic with Exponential Backoff' },

  // Set each session to its state
  { type: 'PreToolUse', sessionId: 'sess-running', toolId: 't1', toolName: 'Read', toolLabel: 'main.ts' },
  { type: 'Stop', sessionId: 'sess-idle' },
  { type: 'PermissionRequest', sessionId: 'sess-attention', toolName: 'Bash' },
  { type: 'PreCompact', sessionId: 'sess-compacting' },
  { type: 'Stale', sessionId: 'sess-stale' },
  { type: 'PreToolUse', sessionId: 'sess-long', toolId: 't2', toolName: 'Edit', toolLabel: 'db/pool.ts' },
];

// Random tool names and labels for continuous simulation
const TOOL_NAMES = ['Read', 'Edit', 'Write', 'Bash', 'Grep', 'Glob', 'Task', 'WebFetch', 'mcp__notion__search'];
const TOOL_LABELS = ['main.ts', 'config.json', 'index.tsx', 'npm test', 'TODO', 'src/**/*.ts', 'refactor', 'docs', 'api.ts'];

function getRandomToolEvent(sessionId: string, toolId: number): { type: string; sessionId: string; toolId: string; toolName: string; toolLabel: string } {
  return {
    type: 'PreToolUse',
    sessionId,
    toolId: `t${toolId}`,
    toolName: TOOL_NAMES[Math.floor(Math.random() * TOOL_NAMES.length)],
    toolLabel: TOOL_LABELS[Math.floor(Math.random() * TOOL_LABELS.length)],
  };
}

// Event cycle: cycles through all states for any session
function getEventCycle(sessionId: string, step: number, cycleIndex: number) {
  const events = [
    // Running with tools (0-3)
    getRandomToolEvent(sessionId, step * 100 + cycleIndex),
    getRandomToolEvent(sessionId, step * 100 + cycleIndex + 1),
    { type: 'PostToolUse', sessionId, toolId: `t${step * 100 + cycleIndex}` },
    { type: 'PostToolUse', sessionId, toolId: `t${step * 100 + cycleIndex + 1}` },
    // Idle (4)
    { type: 'Stop', sessionId },
    // Attention (5)
    { type: 'PermissionRequest', sessionId, toolName: TOOL_NAMES[Math.floor(Math.random() * TOOL_NAMES.length)] },
    // Compacting (6)
    { type: 'PreCompact', sessionId },
    // Stale (7)
    { type: 'Stale', sessionId },
    // Back to running (8)
    { type: 'UserPromptSubmit', sessionId },
  ];
  return events[cycleIndex % events.length];
}

const SIMULATION_INTERVAL_MS = 3000;

export default function App() {
  const { sessions, handleEvent, clearAll, removeSession } = useSessionManager();
  const [isExpanded, setIsExpanded] = useState(false);
  const [bgClass, setBgClass] = useState('');
  const [bgImage, setBgImage] = useState<string | null>(null);
  const [simulationRunning, setSimulationRunning] = useState(false);
  const [showIconPreview, setShowIconPreview] = useState(false);
  const simulationStep = useRef(0);
  const prevSessionCount = useRef(0);
  // Separate drag states for indicator and session list
  const {
    position: indicatorPosition,
    isDragging: isIndicatorDragging,
    wasDragged: indicatorWasDragged,
    handleMouseDown: handleIndicatorMouseDown,
    resetDragState: resetIndicatorDragState,
  } = useDrag();

  const {
    position: listPosition,
    isDragging: isListDragging,
    handleMouseDown: handleListMouseDown,
  } = useDrag({ x: 0, y: 48 }); // Initial offset below indicator (36px + 12px gap)

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

      const step = simulationStep.current;

      // Cycle all sessions through events (offset each session so they're not in sync)
      // Skip sessions that are stale
      sessions.forEach((session, index) => {
        if (session.state === 'stale') return;
        const cycleIndex = (step + index * 3) % 9; // 9 events in cycle, offset by 3
        handleEvent(getEventCycle(session.sessionId, step, cycleIndex));
      });

      simulationStep.current += 1;
    }, SIMULATION_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [simulationRunning, handleEvent, sessions]);

  const handleToggleView = useCallback(() => {
    // Skip toggle if this was a drag gesture, not a click
    if (indicatorWasDragged) {
      resetIndicatorDragState();
      return;
    }
    setIsExpanded(prev => !prev);
  }, [indicatorWasDragged, resetIndicatorDragState]);


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

  if (showIconPreview) {
    return (
      <>
        <IconPreview />
        <button
          onClick={() => setShowIconPreview(false)}
          className="fixed top-5 right-5 px-4 py-2 bg-purple-400 text-white border-none rounded-lg cursor-pointer"
        >
          Back to Prototype
        </button>
      </>
    );
  }

  return (
    <>
      {/* Indicator with its own position */}
      <div
        className="fixed top-[30px] left-1/2 z-[1000] flex flex-col items-center gap-3 w-80"
        style={{
          transform: `translateX(-50%) translate(${indicatorPosition.x}px, ${indicatorPosition.y}px)`,
          cursor: isIndicatorDragging ? 'grabbing' : undefined,
        }}
      >
        <Indicator sessions={sessions} onClick={handleToggleView} onDragStart={handleIndicatorMouseDown} />
      </div>

      {/* Session list with independent position */}
      {isExpanded && sessions.length > 0 && (
        <div
          className="fixed top-[30px] left-1/2 z-[999]"
          style={{
            transform: `translateX(-50%) translate(${listPosition.x}px, ${listPosition.y}px)`,
            cursor: isListDragging ? 'grabbing' : undefined,
          }}
        >
          <SessionList
            sessions={sessions}
            onDragStart={handleListMouseDown}
            onRemoveSession={removeSession}
          />
        </div>
      )}

      <button
        onClick={() => setShowIconPreview(true)}
        className="fixed top-5 right-5 px-4 py-2 bg-white/10 text-white border border-white/20 rounded-lg cursor-pointer"
      >
        Icon Preview
      </button>

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
