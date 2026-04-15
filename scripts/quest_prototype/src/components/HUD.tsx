import { useEffect, useRef, useState } from "react";
import { useQuest } from "../state/quest-context";
import { downloadLog } from "../logging";
import { DreamcallerPortrait } from "./DreamcallerPortrait";
import { DreamcallerPopover } from "./DreamcallerPopover";

/** Duration in ms for the essence count animation. */
const ESSENCE_ANIM_DURATION = 500;

/** Animates a number from one value to another over a duration. */
function useAnimatedNumber(target: number, duration: number): number {
  const [display, setDisplay] = useState(target);
  const displayRef = useRef(target);
  const frameRef = useRef(0);

  useEffect(() => {
    const start = displayRef.current;

    if (start === target) return;

    const delta = target - start;
    const startTime = performance.now();

    function tick(now: number) {
      const elapsed = now - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      const value = Math.round(start + delta * eased);
      displayRef.current = value;
      setDisplay(value);

      if (progress < 1) {
        frameRef.current = requestAnimationFrame(tick);
      }
    }

    frameRef.current = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(frameRef.current);
    };
  }, [target, duration]);

  return display;
}

/** Props for the HUD component. */
interface HudProps {
  onOpenDeckViewer: () => void;
  onOpenDebugScreen: () => void;
  hasDraftData: boolean;
}

/** Persistent HUD bar anchored to the bottom of the viewport. */
export function HUD({ onOpenDeckViewer, onOpenDebugScreen, hasDraftData }: HudProps) {
  const { state } = useQuest();
  const animatedEssence = useAnimatedNumber(
    state.essence,
    ESSENCE_ANIM_DURATION,
  );

  function handleDownloadLog() {
    downloadLog();
  }

  const dreamcallerName = state.dreamcaller?.name ?? null;
  const dreamcallerColor = dreamcallerName !== null ? "#e2e8f0" : "#6b7280";

  return (
    <div
      className="fixed right-0 bottom-0 left-0 z-50 flex items-center justify-between px-3 py-2 md:px-6"
      style={{
        background:
          "linear-gradient(180deg, rgba(10, 6, 18, 0.85) 0%, rgba(10, 6, 18, 0.95) 100%)",
        borderTop: "1px solid rgba(124, 58, 237, 0.3)",
        backdropFilter: "blur(8px)",
      }}
    >
      {/* Left section: essence, deck, dreamcaller */}
      <div className="flex items-center gap-3 md:gap-5">
        {/* Essence counter */}
        <div className="flex items-center gap-1.5">
          <span
            className="text-base md:text-lg"
            style={{ color: "#fbbf24" }}
            aria-label="Essence"
          >
            {"\u25C6"}
          </span>
          <span
            className="text-sm font-bold md:text-base"
            style={{ color: "#fbbf24" }}
          >
            {String(animatedEssence)}
          </span>
          <span className="hidden text-xs opacity-50 lg:inline">Essence</span>
        </div>

        {/* Deck size */}
        <div className="flex items-center gap-1.5">
          <span className="text-sm opacity-70 md:text-base" aria-label="Deck">
            {"\uD83C\uDCCF"}
          </span>
          <span className="text-sm font-bold md:text-base">
            {String(state.deck.length)}
          </span>
          <span className="hidden text-xs opacity-50 lg:inline">Cards</span>
        </div>

        {/* Dreamcaller portrait */}
        <div className="group relative flex items-center gap-1.5">
          <button
            type="button"
            className="flex items-center gap-2 rounded-md px-1 py-0.5 text-left"
            style={{
              color: dreamcallerColor,
            }}
          >
            {state.dreamcaller !== null ? (
              <>
                <DreamcallerPortrait
                  dreamcaller={state.dreamcaller}
                  variant="thumb"
                  style={{ height: 30, width: 30, flexShrink: 0 }}
                />
                <span className="hidden min-w-0 flex-col lg:flex">
                  <span
                    className="max-w-[128px] truncate text-xs font-semibold"
                    style={{ color: dreamcallerColor }}
                  >
                    {state.dreamcaller.name}
                  </span>
                  <span
                    className="max-w-[128px] truncate text-[10px] italic opacity-70"
                    style={{ color: "#cbd5f5" }}
                  >
                    {state.dreamcaller.title}
                  </span>
                </span>
              </>
            ) : (
              <div
                className="flex h-[30px] w-[30px] items-center justify-center rounded-[10px] text-[10px]"
                style={{
                  border: "1px solid rgba(255, 255, 255, 0.14)",
                  background: "rgba(0, 0, 0, 0.35)",
                  color: "#6b7280",
                }}
              >
                {"--"}
              </div>
            )}
          </button>
          {state.dreamcaller !== null && (
            <div
              className="pointer-events-none absolute bottom-full left-0 z-30 mb-3 hidden origin-bottom-left opacity-0 transition-opacity duration-150 group-hover:opacity-100 group-focus-within:opacity-100 lg:block"
            >
              <DreamcallerPopover
                dreamcaller={state.dreamcaller}
                resolvedPackage={state.resolvedPackage}
              />
            </div>
          )}
        </div>

        {/* Dreamsign count */}
        <div className="flex items-center gap-1.5">
          <span
            className="text-sm opacity-70 md:text-base"
            aria-label="Dreamsigns"
          >
            {"\u2728"}
          </span>
          <span className="text-sm font-bold md:text-base">
            {String(state.dreamsigns.length)}
          </span>
          <span className="hidden text-xs opacity-50 lg:inline">Signs</span>
        </div>
      </div>

      {/* Center: completion level */}
      <div className="flex items-center">
        <span className="text-xs font-medium opacity-70 md:text-sm">
          Battle {String(state.completionLevel)}/7
        </span>
      </div>

      {/* Right section: buttons */}
      <div className="flex items-center gap-2 md:gap-3">
        <button
          className="cursor-pointer rounded px-2 py-1 text-xs font-medium transition-colors md:px-3 md:text-sm"
          style={{
            background: "rgba(124, 58, 237, 0.2)",
            border: "1px solid rgba(124, 58, 237, 0.4)",
            color: "#c084fc",
          }}
          onClick={onOpenDeckViewer}
        >
          <span className="lg:hidden">{"\uD83C\uDCCF"}</span>
          <span className="hidden lg:inline">View Deck</span>
        </button>
        {hasDraftData && (
          <button
            className="cursor-pointer rounded px-2 py-1 text-xs font-medium transition-colors md:px-3 md:text-sm"
            style={{
              background: "rgba(239, 68, 68, 0.15)",
              border: "1px solid rgba(239, 68, 68, 0.3)",
              color: "#f87171",
            }}
            onClick={onOpenDebugScreen}
          >
            <span className="lg:hidden">{"\uD83D\uDC1B"}</span>
            <span className="hidden lg:inline">Debug</span>
          </button>
        )}
        <button
          className="cursor-pointer rounded px-2 py-1 text-xs font-medium transition-colors md:px-3 md:text-sm"
          style={{
            background: "rgba(212, 160, 23, 0.15)",
            border: "1px solid rgba(212, 160, 23, 0.3)",
            color: "#fbbf24",
          }}
          onClick={handleDownloadLog}
        >
          <span className="lg:hidden">{"\u2B73"}</span>
          <span className="hidden lg:inline">Download Log</span>
        </button>
      </div>
    </div>
  );
}
