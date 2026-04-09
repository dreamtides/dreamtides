import { useEffect, useRef, useState } from "react";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { downloadLog } from "../logging";
import { TIDE_COLORS } from "../data/card-database";

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
  onOpenDeckEditor: () => void;
}

/** Persistent HUD bar anchored to the bottom of the viewport. */
export function HUD({ onOpenDeckEditor }: HudProps) {
  const { state } = useQuest();
  const config = useQuestConfig();
  const animatedEssence = useAnimatedNumber(
    state.essence,
    ESSENCE_ANIM_DURATION,
  );

  function handleDownloadLog() {
    downloadLog();
  }

  const dreamcallerName = state.dreamcaller?.name ?? null;
  const dreamcallerTide = state.dreamcaller?.tides[0] ?? null;
  const dreamcallerColor =
    dreamcallerTide !== null ? TIDE_COLORS[dreamcallerTide] : "#6b7280";

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

        {/* Pool size */}
        <div className="flex items-center gap-1.5">
          <span className="text-sm opacity-70 md:text-base" aria-label="Pool">
            {"\uD83C\uDCCF"}
          </span>
          <span className="text-sm font-bold md:text-base">
            Pool: {String(state.pool.length)}
          </span>
        </div>

        {/* Deck size with min/max */}
        <div className="flex items-center gap-1.5">
          <span className="text-sm opacity-70 md:text-base" aria-label="Deck">
            {"\uD83C\uDCCF"}
          </span>
          <span className="text-sm font-bold md:text-base">
            Deck: {String(state.deck.length)}/{String(config.minimumDeckSize)}-{String(config.maximumDeckSize)}
          </span>
        </div>

        {/* Dreamcaller portrait */}
        <div className="flex items-center gap-1.5">
          <div
            className="flex h-6 w-6 items-center justify-center rounded md:h-7 md:w-7"
            style={{
              border: `2px solid ${dreamcallerColor}`,
              background: "rgba(0, 0, 0, 0.4)",
            }}
            aria-label="Dreamcaller"
          >
            {dreamcallerName !== null ? (
              <span
                className="text-[10px] font-bold md:text-xs"
                style={{ color: dreamcallerColor }}
              >
                {dreamcallerName.charAt(0)}
              </span>
            ) : (
              <span className="text-[10px] opacity-30">{"--"}</span>
            )}
          </div>
          <span
            className="hidden max-w-[80px] truncate text-xs font-medium lg:inline"
            style={{ color: dreamcallerColor }}
          >
            {dreamcallerName ?? "None"}
          </span>
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
          onClick={onOpenDeckEditor}
        >
          <span className="lg:hidden">{"\uD83C\uDCCF"}</span>
          <span className="hidden lg:inline">Deck</span>
        </button>
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
