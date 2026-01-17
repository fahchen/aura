import React, { useState } from 'react';

interface ControlsProps {
  onToggleView: () => void;
  onRunSimulation: () => void;
  onStopSimulation: () => void;
  onAddSession: (cwd: string) => void;
  onSetBackground: (bg: string) => void;
  simulationRunning: boolean;
}

export function Controls({
  onToggleView,
  onRunSimulation,
  onStopSimulation,
  onAddSession,
  onSetBackground,
  simulationRunning,
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
    <div className="controls-panel">
      <h3 className="mb-4 text-gray-800">Aura Prototype Controls</h3>

      <div className="flex gap-2 mb-3 flex-wrap items-center">
        <label className="text-sm text-gray-600">Background:</label>
        <button
          onClick={() => onSetBackground('bg-white')}
          className="px-4 py-2 bg-indigo-600 text-white border-none rounded-md cursor-pointer text-sm transition-colors hover:bg-indigo-700"
        >
          White
        </button>
        <button
          onClick={() => onSetBackground('bg-dark')}
          className="px-4 py-2 bg-indigo-600 text-white border-none rounded-md cursor-pointer text-sm transition-colors hover:bg-indigo-700"
        >
          Dark
        </button>
        <button
          onClick={() => onSetBackground('random')}
          className="px-4 py-2 bg-indigo-600 text-white border-none rounded-md cursor-pointer text-sm transition-colors hover:bg-indigo-700"
        >
          Random
        </button>
      </div>

      <div className="flex gap-2 mb-3 flex-wrap items-center">
        <button
          onClick={onToggleView}
          className="px-4 py-2 bg-indigo-600 text-white border-none rounded-md cursor-pointer text-sm transition-colors hover:bg-indigo-700"
        >
          Toggle View
        </button>
      </div>

      <div className="flex gap-2 mb-3 flex-wrap items-center">
        {simulationRunning ? (
          <button
            onClick={onStopSimulation}
            className="px-4 py-2 bg-indigo-600 text-white border-none rounded-md cursor-pointer text-sm transition-colors hover:bg-indigo-700"
          >
            Stop Simulation
          </button>
        ) : (
          <button
            onClick={onRunSimulation}
            className="px-4 py-2 bg-indigo-600 text-white border-none rounded-md cursor-pointer text-sm transition-colors hover:bg-indigo-700"
          >
            Run Simulation
          </button>
        )}
      </div>

      <div className="flex gap-2 mb-3 flex-wrap items-center">
        <input
          type="text"
          value={cwdInput}
          onChange={e => setCwdInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Session CWD"
          className="flex-1 min-w-[150px] px-3 py-2 border border-gray-300 rounded-md text-sm"
        />
        <button
          onClick={handleAddSession}
          className="px-4 py-2 bg-indigo-600 text-white border-none rounded-md cursor-pointer text-sm transition-colors hover:bg-indigo-700"
        >
          Add Session
        </button>
      </div>
    </div>
  );
}
