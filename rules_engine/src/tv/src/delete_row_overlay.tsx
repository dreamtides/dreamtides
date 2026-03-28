import { useEffect, useRef, useState } from "react";
import type { FUniver } from "@univerjs/core/facade";
import { createLogger } from "./logger_frontend";

import "./styles/delete_row_overlay_styles.css";

const logger = createLogger("tv.ui.delete_overlay");

const LEFT_EDGE_THRESHOLD_PX = 80;
const BUTTON_HEIGHT_PX = 20;

interface DeleteRowOverlayProps {
  containerRef: React.RefObject<HTMLDivElement | null>;
  univerAPI: FUniver | null;
  onDeleteRow?: (sheetId: string, displayRowIndex: number) => void;
  activeSheetId: string;
}

export function DeleteRowOverlay({
  containerRef,
  univerAPI,
  onDeleteRow,
  activeSheetId,
}: DeleteRowOverlayProps) {
  const [buttonPosition, setButtonPosition] = useState<{
    x: number;
    y: number;
  } | null>(null);
  const hoveredRowRef = useRef<number | null>(null);
  const mouseXRef = useRef(0);
  const rafRef = useRef<number | null>(null);

  useEffect(() => {
    if (!univerAPI) return;

    const sub = univerAPI.addEvent(
      univerAPI.Event.CellHover,
      (evt: { row: number; column: number }) => {
        hoveredRowRef.current = evt.row;

        if (mouseXRef.current < LEFT_EDGE_THRESHOLD_PX && evt.row > 0) {
          setButtonPosition((prev) => (prev ? { x: prev.x, y: prev.y } : null));
        }
      },
    );

    return () => {
      sub.dispose();
    };
  }, [univerAPI]);

  useEffect(() => {
    hoveredRowRef.current = null;
    setButtonPosition(null);
  }, [activeSheetId]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = container.getBoundingClientRect();
      const localX = e.clientX - rect.left;
      const localY = e.clientY - rect.top;
      mouseXRef.current = localX;

      if (rafRef.current !== null) return;

      rafRef.current = requestAnimationFrame(() => {
        rafRef.current = null;
        const row = hoveredRowRef.current;

        if (mouseXRef.current >= LEFT_EDGE_THRESHOLD_PX || row === null || row <= 0) {
          setButtonPosition(null);
          return;
        }

        setButtonPosition({ x: 4, y: localY - BUTTON_HEIGHT_PX / 2 });
      });
    };

    const handleMouseLeave = () => {
      setButtonPosition(null);
      hoveredRowRef.current = null;
    };

    container.addEventListener("mousemove", handleMouseMove);
    container.addEventListener("mouseleave", handleMouseLeave);

    return () => {
      container.removeEventListener("mousemove", handleMouseMove);
      container.removeEventListener("mouseleave", handleMouseLeave);
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
      }
    };
  }, [containerRef]);

  const handleClick = () => {
    const row = hoveredRowRef.current;
    if (row === null || row <= 0 || !onDeleteRow) return;

    const displayRowIndex = row - 1;
    logger.info("Delete button clicked", { activeSheetId, displayRowIndex });
    onDeleteRow(activeSheetId, displayRowIndex);
    setButtonPosition(null);
    hoveredRowRef.current = null;
  };

  return (
    <div className="tv-delete-row-overlay">
      {buttonPosition && (
        <button
          className="tv-delete-row-button"
          style={{ left: buttonPosition.x, top: buttonPosition.y }}
          onClick={handleClick}
        >
          Delete
        </button>
      )}
    </div>
  );
}
