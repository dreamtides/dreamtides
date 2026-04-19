import type { ReactNode } from "react";
import { useEffect, useRef, useState } from "react";

/**
 * Tiny popover button used to tuck secondary metadata out of the terse
 * status bar (staggered-parity layout). Click or focus toggles an
 * anchored popover; outside-click and Escape close it. No external deps.
 */
export function HoverInfoButton({
  label,
  icon = "i",
  align = "end",
  dataTestId,
  children,
}: {
  label: string;
  icon?: ReactNode;
  align?: "start" | "end";
  dataTestId?: string;
  children: ReactNode;
}) {
  const [isOpen, setIsOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const popoverRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    function handleClick(event: MouseEvent): void {
      const target = event.target as Node | null;
      if (target === null) {
        return;
      }
      if (
        (buttonRef.current !== null && buttonRef.current.contains(target)) ||
        (popoverRef.current !== null && popoverRef.current.contains(target))
      ) {
        return;
      }
      setIsOpen(false);
    }
    function handleKey(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        setIsOpen(false);
      }
    }
    window.addEventListener("click", handleClick);
    window.addEventListener("keydown", handleKey);
    return () => {
      window.removeEventListener("click", handleClick);
      window.removeEventListener("keydown", handleKey);
    };
  }, [isOpen]);

  const alignClass = align === "end" ? "right-0" : "left-0";

  return (
    <span className="relative inline-block">
      <button
        ref={buttonRef}
        type="button"
        aria-label={label}
        aria-haspopup="dialog"
        aria-expanded={isOpen}
        title={label}
        data-battle-hover-info-button={dataTestId ?? ""}
        className="inline-flex h-5 w-5 items-center justify-center rounded-full border border-[var(--color-border)] bg-[var(--color-surface-light)] text-[10px] font-bold text-[var(--color-text-dim)] transition hover:border-[var(--color-primary-light)] hover:text-[var(--color-primary-light)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-primary-light)]"
        onClick={() => setIsOpen((v) => !v)}
      >
        {icon}
      </button>
      {isOpen ? (
        <div
          ref={popoverRef}
          role="dialog"
          aria-label={label}
          data-battle-hover-info-popover={dataTestId ?? ""}
          className={`absolute top-full mt-1 ${alignClass} z-40 min-w-[220px] max-w-[320px] rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] p-3 text-[11px] text-[var(--color-text)] shadow-xl shadow-slate-950/60`}
          onClick={(event) => event.stopPropagation()}
        >
          {children}
        </div>
      ) : null}
    </span>
  );
}
