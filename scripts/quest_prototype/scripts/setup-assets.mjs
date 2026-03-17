import { readFileSync, mkdirSync, rmSync, symlinkSync, copyFileSync, readdirSync, existsSync } from "node:fs";
import { writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { resolve, join } from "node:path";
import { homedir } from "node:os";
import { execSync } from "node:child_process";
import { parse } from "smol-toml";

const ROOT = resolve(import.meta.dirname, "..");
const PROJECT_ROOT = resolve(ROOT, "..", "..");
const IMAGE_CACHE_DIR = join(homedir(), "Library", "Caches", "io.github.dreamtides.tv", "image_cache");

const PUBLIC_DIR = join(ROOT, "public");
const CARDS_DIR = join(PUBLIC_DIR, "cards");
const TIDES_DIR = join(PUBLIC_DIR, "tides");
const JSON_PATH = join(PUBLIC_DIR, "card-data.json");

/**
 * Find the main git worktree root for resolving untracked assets. Falls back
 * to the default project root (../../ from quest_prototype).
 */
function findMainWorktreeRoot() {
  try {
    const gitCommonDir = execSync("git rev-parse --git-common-dir", {
      cwd: ROOT,
      encoding: "utf8",
    }).trim();
    const absGitDir = resolve(ROOT, gitCommonDir);
    // The main worktree's .git directory is at <project_root>/.git
    return resolve(absGitDir, "..");
  } catch {
    return PROJECT_ROOT;
  }
}

/**
 * Resolve a path that may contain untracked files. Tries the local project
 * root first, then the main worktree root.
 */
function resolveAssetPath(...segments) {
  const localPath = join(PROJECT_ROOT, ...segments);
  if (existsSync(localPath)) {
    return localPath;
  }
  const mainRoot = findMainWorktreeRoot();
  const mainPath = join(mainRoot, ...segments);
  if (existsSync(mainPath)) {
    return mainPath;
  }
  return localPath;
}

/**
 * Convert a kebab-case string to camelCase.
 */
function kebabToCamel(str) {
  return str.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}

/**
 * Convert a TOML card record to its JSON representation with camelCase keys.
 * Spark normalization: "" or missing becomes null; "*" (variable spark)
 * becomes null; integer values are preserved.
 */
function transformCard(card) {
  const result = {};
  for (const [key, value] of Object.entries(card)) {
    const camelKey = kebabToCamel(key);
    if (camelKey === "spark" || camelKey === "energyCost") {
      result[camelKey] = value === "" || value === "*" ? null : value;
    } else {
      result[camelKey] = value;
    }
  }
  if (!("spark" in result)) {
    result.spark = null;
  }
  if (!("subtype" in result) || result.subtype == null) {
    result.subtype = "";
  }
  return result;
}

/**
 * Compute the SHA-256 hash of the Shutterstock URL for a given image number.
 */
function imageHash(imageNumber) {
  const url = `https://www.shutterstock.com/image-illustration/-260nw-${imageNumber}.jpg`;
  return createHash("sha256").update(url).digest("hex");
}

/**
 * Clean and recreate a directory for idempotent runs.
 */
function recreateDir(dir) {
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
}

function main() {
  const tomlPath = resolveAssetPath("client", "Assets", "StreamingAssets", "Tabula", "rendered-cards.toml");
  const tideIconsDir = resolveAssetPath("client", "Assets", "ThirdParty", "GameAssets", "Tides");

  console.log("Parsing rendered-cards.toml...");
  const tomlContent = readFileSync(tomlPath, "utf8");
  const parsed = parse(tomlContent);
  const allCards = parsed.cards;

  if (!Array.isArray(allCards)) {
    throw new Error("Expected [[cards]] array in TOML file");
  }

  console.log(`Found ${allCards.length} total cards`);

  // Filter out Special-rarity cards
  const cards = allCards.filter((c) => c.rarity !== "Special");
  console.log(`Filtered to ${cards.length} non-Special cards`);

  // Transform to camelCase JSON
  const jsonCards = cards.map(transformCard);

  // Write card-data.json
  mkdirSync(PUBLIC_DIR, { recursive: true });
  writeFileSync(JSON_PATH, JSON.stringify(jsonCards, null, 2) + "\n");
  console.log(`Wrote ${jsonCards.length} cards to card-data.json`);

  // Create card image symlinks
  recreateDir(CARDS_DIR);
  let linked = 0;
  let missing = 0;

  for (const card of jsonCards) {
    const hash = imageHash(card.imageNumber);
    const cachePath = join(IMAGE_CACHE_DIR, hash);
    const symlinkPath = join(CARDS_DIR, `${card.cardNumber}.webp`);

    if (existsSync(cachePath)) {
      symlinkSync(cachePath, symlinkPath);
      linked++;
    } else {
      console.warn(`  Warning: missing cache file for card ${card.cardNumber} (${card.name}): ${hash}`);
      missing++;
    }
  }

  console.log(`Linked ${linked} of ${jsonCards.length} card images (${missing} missing)`);

  // Copy tide icon PNGs
  recreateDir(TIDES_DIR);

  if (!existsSync(tideIconsDir)) {
    console.warn("Warning: tide icons directory not found, skipping tide icon copy");
  } else {
    const tideFiles = readdirSync(tideIconsDir).filter(
      (f) => f.endsWith(".png") && !f.endsWith(".meta")
    );

    for (const file of tideFiles) {
      copyFileSync(join(tideIconsDir, file), join(TIDES_DIR, file));
    }

    console.log(`Copied ${tideFiles.length} tide icons to public/tides/`);
  }

  console.log("Asset setup complete.");
}

main();
