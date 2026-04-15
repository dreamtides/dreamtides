export type CardSizePreset = "small" | "medium" | "large";

const CARD_SIZE_STORAGE_KEY = "quest-prototype-card-size";

export const SIZE_PRESETS: Readonly<
  Record<CardSizePreset, { columns: string; gap: string; label: string }>
> = {
  small: {
    columns: "repeat(auto-fill, minmax(112px, 1fr))",
    gap: "0.375rem",
    label: "S",
  },
  medium: {
    columns: "repeat(auto-fill, minmax(160px, 1fr))",
    gap: "0.5rem",
    label: "M",
  },
  large: {
    columns: "repeat(auto-fill, minmax(220px, 1fr))",
    gap: "0.75rem",
    label: "L",
  },
};

export function getPersistedCardSize(
  fallback: CardSizePreset,
): CardSizePreset {
  try {
    const stored = localStorage.getItem(CARD_SIZE_STORAGE_KEY);
    if (stored === "small" || stored === "medium" || stored === "large") {
      return stored;
    }
  } catch {
    return fallback;
  }

  return fallback;
}

export function persistCardSize(size: CardSizePreset): void {
  try {
    localStorage.setItem(CARD_SIZE_STORAGE_KEY, size);
  } catch {
    return;
  }
}
