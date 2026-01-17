import { useState, useCallback, useEffect, useRef } from 'react';

interface Position {
  x: number;
  y: number;
}

interface UseDragResult {
  position: Position;
  isDragging: boolean;
  wasDragged: boolean;
  handleMouseDown: (e: React.MouseEvent) => void;
  resetDragState: () => void;
}

// Minimum movement threshold to consider it a drag (pixels)
const DRAG_THRESHOLD = 3;

export function useDrag(initialPosition: Position = { x: 0, y: 0 }): UseDragResult {
  const [position, setPosition] = useState<Position>(initialPosition);
  const [isDragging, setIsDragging] = useState(false);
  const [wasDragged, setWasDragged] = useState(false);
  const dragStart = useRef<Position>({ x: 0, y: 0 });
  const positionStart = useRef<Position>({ x: 0, y: 0 });
  const hasMoved = useRef(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
    setWasDragged(false);
    hasMoved.current = false;
    dragStart.current = { x: e.clientX, y: e.clientY };
    positionStart.current = { ...position };
  }, [position]);

  const resetDragState = useCallback(() => {
    setWasDragged(false);
  }, []);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const dx = e.clientX - dragStart.current.x;
      const dy = e.clientY - dragStart.current.y;

      // Check if movement exceeds threshold
      if (!hasMoved.current && (Math.abs(dx) > DRAG_THRESHOLD || Math.abs(dy) > DRAG_THRESHOLD)) {
        hasMoved.current = true;
      }

      setPosition({
        x: positionStart.current.x + dx,
        y: positionStart.current.y + dy,
      });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      // Set wasDragged if mouse actually moved beyond threshold
      if (hasMoved.current) {
        setWasDragged(true);
      }
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging]);

  return { position, isDragging, wasDragged, handleMouseDown, resetDragState };
}
