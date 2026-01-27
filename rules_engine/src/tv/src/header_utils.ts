/** Known acronyms that should be fully uppercased in display headers. */
const HEADER_ACRONYMS = new Set(["id", "fx", "url", "uuid", "hp", "ui"]);

/**
 * Converts a TOML key to a display-friendly header name.
 * Splits on hyphens and underscores, title-cases each word,
 * and fully uppercases known acronyms.
 */
export function formatHeaderForDisplay(key: string): string {
  return key
    .split(/[-_]/)
    .map((word) =>
      HEADER_ACRONYMS.has(word.toLowerCase())
        ? word.toUpperCase()
        : word.charAt(0).toUpperCase() + word.slice(1).toLowerCase(),
    )
    .join(" ");
}

export function getColumnLetter(index: number): string {
  let result = "";
  let n = index;
  while (n >= 0) {
    result = String.fromCharCode((n % 26) + 65) + result;
    n = Math.floor(n / 26) - 1;
  }
  return result;
}
