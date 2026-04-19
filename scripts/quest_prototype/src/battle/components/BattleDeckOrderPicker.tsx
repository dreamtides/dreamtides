import { useEffect, useRef, useState } from "react";
import type { BattleMutableState, BattleSide } from "../types";
import { formatSideLabel } from "../ui/format";

export type BattleDeckOrderPickerScope = "top-N" | "full";

export function BattleDeckOrderPicker({
  initialOrder,
  onCancel,
  onConfirm,
  scopeLabel,
  side,
  state,
}: {
  initialOrder: readonly string[];
  onCancel: () => void;
  onConfirm: (order: readonly string[]) => void;
  scopeLabel: BattleDeckOrderPickerScope;
  side: BattleSide;
  state: BattleMutableState;
}) {
  const [draftOrder, setDraftOrder] = useState<readonly string[]>(() => [...initialOrder]);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const previouslyFocusedRef = useRef<HTMLElement | null>(null);

  function handleMove(index: number, direction: -1 | 1): void {
    setDraftOrder((current) => {
      const nextIndex = index + direction;
      if (nextIndex < 0 || nextIndex >= current.length) {
        return current;
      }
      const next = [...current];
      const tmp = next[index];
      next[index] = next[nextIndex];
      next[nextIndex] = tmp;
      return next;
    });
  }

  function buildFinalOrder(): readonly string[] {
    return scopeLabel === "full"
      ? [...draftOrder]
      : [...draftOrder, ...state.sides[side].deck.slice(draftOrder.length)];
  }

  function handleConfirm(): void {
    onConfirm(buildFinalOrder());
  }

  useEffect(() => {
    // bug-099: focus management — move focus into the dialog on mount and
    // restore it on unmount.
    previouslyFocusedRef.current = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    const container = containerRef.current;
    if (container !== null) {
      const confirmButton = container.querySelector<HTMLButtonElement>(
        'button[data-battle-deck-order-action="confirm"]',
      );
      confirmButton?.focus();
    }
    return () => {
      previouslyFocusedRef.current?.focus();
    };
  }, []);

  useEffect(() => {
    // bug-100: scope Enter-to-confirm to the picker container so default-
    // button activations elsewhere in the tree don't trigger confirm, and so
    // pressing Enter on Move Up/Down buttons activates the button without
    // also closing the picker.
    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        onCancel();
        return;
      }
      if (event.key !== "Enter") {
        return;
      }
      const container = containerRef.current;
      if (container === null) {
        return;
      }
      const target = event.target;
      if (target instanceof HTMLElement && container.contains(target)) {
        if (
          target instanceof HTMLButtonElement ||
          target instanceof HTMLInputElement ||
          target instanceof HTMLTextAreaElement
        ) {
          // Let the focused interactive element handle Enter natively.
          return;
        }
      }
      onConfirm(buildFinalOrder());
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [draftOrder, onCancel, onConfirm, scopeLabel, side, state]);

  return (
    <div
      className="fixed inset-0 z-50 bg-slate-950/80 p-3 backdrop-blur"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          onCancel();
        }
      }}
    >
      <div
        ref={containerRef}
        // bug-099: dialog semantics + labelled heading so screen readers
        // announce the picker correctly.
        role="dialog"
        aria-modal="true"
        aria-labelledby="battle-deck-order-picker-title"
        tabIndex={-1}
        data-battle-deck-order-picker=""
        data-battle-deck-order-scope={scopeLabel}
        data-battle-deck-order-side={side}
        className="pointer-events-auto mx-auto flex max-h-full w-full max-w-2xl flex-col gap-4 overflow-hidden rounded-[2rem] border border-violet-300/25 bg-[linear-gradient(180deg,_rgba(7,10,18,0.98)_0%,_rgba(11,17,30,0.96)_100%)] p-5 shadow-2xl shadow-slate-950/70"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="border-b border-slate-800 pb-3">
          <p className="text-xs font-semibold uppercase tracking-[0.28em] text-violet-300">
            Deck Order
          </p>
          <h3
            id="battle-deck-order-picker-title"
            className="mt-2 text-lg font-semibold text-white"
          >
            Reorder {formatScopeLabel(scopeLabel)} of {formatSideLabel(side)} Deck
          </h3>
          <p className="mt-1 text-sm text-slate-400">
            Use Move Up / Move Down to set a new deck order. Confirm commits a single history entry.
          </p>
        </header>
        <ol className="flex max-h-[60vh] flex-col gap-2 overflow-y-auto pr-1">
          {draftOrder.length === 0 ? (
            <li className="rounded-2xl border border-dashed border-slate-700 bg-slate-950/60 px-4 py-6 text-center text-sm text-slate-400">
              No cards to reorder.
            </li>
          ) : (
            draftOrder.map((battleCardId, index) => {
              const instance = state.cardInstances[battleCardId];
              const name = instance?.definition.name ?? "Card";
              const subtype = instance?.definition.subtype ?? "";
              const printedSpark = instance?.definition.printedSpark ?? 0;
              const isFirst = index === 0;
              const isLast = index === draftOrder.length - 1;
              return (
                <li
                  key={battleCardId}
                  data-battle-deck-order-slot={String(index)}
                  data-battle-deck-order-card={battleCardId}
                  className="flex items-center gap-3 rounded-2xl border border-slate-800 bg-slate-900/75 px-3 py-2"
                >
                  <span className="rounded-full border border-slate-700 bg-slate-950/75 px-2 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-300">
                    {String(index + 1)}
                  </span>
                  <div className="flex flex-1 flex-col">
                    <span className="text-sm font-semibold text-white">{name}</span>
                    <span className="text-xs text-slate-400">
                      {subtype} &middot; Spark {String(printedSpark)}
                    </span>
                  </div>
                  <div className="flex gap-2">
                    <button
                      type="button"
                      data-battle-deck-order-action="move-up"
                      disabled={isFirst}
                      className={createButtonClassName(!isFirst)}
                      onClick={() => handleMove(index, -1)}
                    >
                      Move Up
                    </button>
                    <button
                      type="button"
                      data-battle-deck-order-action="move-down"
                      disabled={isLast}
                      className={createButtonClassName(!isLast)}
                      onClick={() => handleMove(index, 1)}
                    >
                      Move Down
                    </button>
                  </div>
                </li>
              );
            })
          )}
        </ol>
        <footer className="flex flex-wrap justify-end gap-2 border-t border-slate-800 pt-3">
          <button
            type="button"
            data-battle-deck-order-action="cancel"
            className={createButtonClassName(true)}
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            data-battle-deck-order-action="confirm"
            className={createButtonClassName(true)}
            onClick={handleConfirm}
          >
            Confirm
          </button>
        </footer>
      </div>
    </div>
  );
}

function createButtonClassName(isEnabled: boolean): string {
  return [
    "rounded-full border px-3 py-2 text-xs font-semibold uppercase tracking-[0.18em] transition",
    isEnabled
      ? "border-violet-300/45 bg-violet-400/10 text-violet-50 hover:bg-violet-400/20"
      : "cursor-not-allowed border-slate-800 bg-slate-900/70 text-slate-600",
  ].join(" ");
}

function formatScopeLabel(scope: BattleDeckOrderPickerScope): string {
  return scope === "full" ? "Full" : "Revealed";
}
