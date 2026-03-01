/** Known acronyms that should be fully uppercased in display headers. */
const HEADER_ACRONYMS = new Set(["id", "fx", "url", "uuid", "hp", "ui"]);

/**
 * Converts a TOML key to a display-friendly header name.
 * Splits on hyphens and underscores, title-cases each word,
 * and fully uppercases known acronyms.
 */
export function formatHeaderForDisplay(key: string): string {
  // Handle expanded array column keys like "resonance[0]" → "Resonance 1"
  const arrayMatch = key.match(/^(.+)\[(\d+)\]$/);
  if (arrayMatch) {
    const baseFormatted = formatHeaderForDisplay(arrayMatch[1]);
    return `${baseFormatted} ${parseInt(arrayMatch[2], 10) + 1}`;
  }

  return key
    .split(/[-_]/)
    .map((word) =>
      HEADER_ACRONYMS.has(word.toLowerCase())
        ? word.toUpperCase()
        : word.charAt(0).toUpperCase() + word.slice(1).toLowerCase(),
    )
    .join(" ");
}

/**
 * Finds all header indices that match a column config key.
 *
 * If the key matches a header exactly, returns that single index.
 * Otherwise, checks if any headers are array expansions of the key
 * (e.g., key "resonance" matches "resonance[0]", "resonance[1]", etc.).
 */
export function findMatchingHeaderIndices(
  headers: string[],
  configKey: string,
): number[] {
  const exactIndex = headers.indexOf(configKey);
  if (exactIndex !== -1) {
    return [exactIndex];
  }

  // Check for array column matches: configKey "resonance" matches "resonance[N]"
  const prefix = configKey + "[";
  const indices: number[] = [];
  for (let i = 0; i < headers.length; i++) {
    if (headers[i].startsWith(prefix) && headers[i].endsWith("]")) {
      indices.push(i);
    }
  }
  return indices;
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
