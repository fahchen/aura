import React, { useState } from 'react';

interface ControlsProps {
  onToggleView: () => void;
  onRunSimulation: () => void;
  onStopSimulation: () => void;
  onAddSession: (cwd: string) => void;
  onSetBackground: (bg: string) => void;
  onToggleStyle: () => void;
  simulationRunning: boolean;
  listStyle: 'card' | 'full-width';
}

export function Controls({
  onToggleView,
  onRunSimulation,
  onStopSimulation,
  onAddSession,
  onSetBackground,
  onToggleStyle,
  simulationRunning,
  listStyle,
}: ControlsProps) {
  const [cwdInput, setCwdInput] = useState('/Users/dev/my-project');

  const handleAddSession = () => {
    if (cwdInput.trim()) {
      onAddSession(cwdInput.trim());
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleAddSession();
    }
  };

  return (
    <div className="controls">
      <h3>Aura Prototype Controls</h3>

      <div className="control-group">
        <label>Background:</label>
        <button onClick={() => onSetBackground('bg-white')}>White</button>
        <button onClick={() => onSetBackground('bg-dark')}>Dark</button>
        <button onClick={() => onSetBackground('random')}>Random</button>
      </div>

      <div className="control-group">
        <button onClick={onToggleView}>Toggle View</button>
        <button onClick={onToggleStyle}>
          Style: {listStyle === 'card' ? 'Card' : 'Full'}
        </button>
      </div>

      <div className="control-group">
        {simulationRunning ? (
          <button onClick={onStopSimulation}>Stop Simulation</button>
        ) : (
          <button onClick={onRunSimulation}>Run Simulation</button>
        )}
      </div>

      <div className="control-group">
        <input
          type="text"
          value={cwdInput}
          onChange={e => setCwdInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Session CWD"
        />
        <button onClick={handleAddSession}>Add Session</button>
      </div>
    </div>
  );
}
