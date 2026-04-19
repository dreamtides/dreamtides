import type { MouseEvent } from "react";
import {
  selectCardHasCopiedMarker,
  selectCardHasPreventedMarker,
} from "../state/selectors";
import { formatNoteExpiryChipHint } from "../state/notes-utils";
import type { BattleCardInstance, BattleCardNote } from "../types";

const NOTE_CHIP_LIMIT = 3;
const NOTE_TEXT_TRUNCATION = 16;

export function BattleCardBadges({
  instance,
  onNoteClick,
}: {
  instance: BattleCardInstance;
  onNoteClick?: (noteId: string) => void;
}) {
  const visibleNotes = instance.notes.slice(0, NOTE_CHIP_LIMIT);
  const overflowCount = instance.notes.length - visibleNotes.length;

  return (
    <>
      {selectCardHasPreventedMarker(instance) ? (
        <MarkerChip kind="prevented" />
      ) : null}
      {selectCardHasCopiedMarker(instance) ? (
        <MarkerChip kind="copied" />
      ) : null}
      {visibleNotes.map((note) => (
        <NoteChip
          key={note.noteId}
          note={note}
          onClick={onNoteClick}
        />
      ))}
      {overflowCount > 0 ? (
        <span
          data-battle-card-note="overflow"
          aria-label={`${String(overflowCount)} more notes`}
          className={chipBaseClassName("border-slate-400/60 text-slate-100")}
        >
          +{String(overflowCount)} more
        </span>
      ) : null}
    </>
  );
}

function MarkerChip({
  kind,
}: {
  kind: "prevented" | "copied";
}) {
  if (kind === "prevented") {
    return (
      <span
        data-battle-card-marker="prevented"
        aria-label="Prevented"
        className={chipBaseClassName("border-red-300/70 text-red-100")}
      >
        <ShieldIcon />
        <span>Prevented</span>
      </span>
    );
  }

  return (
    <span
      data-battle-card-marker="copied"
      aria-label="Copied"
      className={chipBaseClassName("border-sky-300/70 text-sky-100")}
    >
      <DuplicateIcon />
      <span>Copied</span>
    </span>
  );
}

function NoteChip({
  note,
  onClick,
}: {
  note: BattleCardNote;
  onClick?: (noteId: string) => void;
}) {
  const { noteId, text } = note;
  const truncated = text.length <= NOTE_TEXT_TRUNCATION
    ? text
    : `${text.slice(0, NOTE_TEXT_TRUNCATION - 1)}\u2026`;
  // bug-107: show a compact expiry hint (e.g. "T4") when the note is scoped
  // to a start-of-turn trigger.
  const expiryHint = formatNoteExpiryChipHint(note);

  function handleClick(event: MouseEvent<HTMLSpanElement>): void {
    if (onClick === undefined) {
      return;
    }
    event.stopPropagation();
    onClick(noteId);
  }

  return (
    <span
      data-battle-card-note={noteId}
      data-battle-card-note-expiry={note.expiry.kind}
      aria-label={`Note: ${text}`}
      title={text}
      role={onClick === undefined ? undefined : "button"}
      onClick={handleClick}
      className={chipBaseClassName("border-amber-300/70 text-amber-100")}
    >
      {truncated}
      {expiryHint === null ? null : (
        <span
          data-battle-card-note-expiry-hint=""
          className="ml-1 rounded-full border border-amber-300/60 px-1 text-[0.5rem] font-semibold uppercase tracking-[0.12em]"
        >
          {expiryHint}
        </span>
      )}
    </span>
  );
}

function ShieldIcon() {
  return (
    <svg
      aria-hidden="true"
      width="10"
      height="10"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M8 1.5L3 3.5v4.5c0 3 2.1 5.3 5 6 2.9-0.7 5-3 5-6V3.5L8 1.5z" />
    </svg>
  );
}

function DuplicateIcon() {
  return (
    <svg
      aria-hidden="true"
      width="10"
      height="10"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect x="3" y="3" width="8" height="8" rx="1" />
      <rect x="6" y="6" width="8" height="8" rx="1" />
    </svg>
  );
}

function chipBaseClassName(accent: string): string {
  return [
    "inline-flex items-center gap-1 rounded-full border px-1.5 py-0.5 text-[0.625rem]",
    accent,
  ].join(" ");
}
