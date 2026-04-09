import { readFileSync } from "node:fs";

const path = process.argv[2];

if (!path) {
  throw new Error("Usage: node src/qa/analyze-quest-log.mjs /path/to/quest-log.jsonl");
}

const events = readFileSync(path, "utf8")
  .trim()
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line));

function requireEvent(name) {
  const event = events.find((entry) => entry.event === name);
  if (!event) throw new Error(`Missing event: ${name}`);
  return event;
}

const selected = requireEvent("starting_tide_selected");
const deck = requireEvent("starting_deck_initialized");
const quest = requireEvent("quest_started");

if (deck.totalDeckSize !== 30) {
  throw new Error(`Expected starting deck size 30; found ${deck.totalDeckSize}`);
}

for (const key of ["starterCardNumbers", "tideCardNumbers", "neutralCardNumbers"]) {
  if (!Array.isArray(deck[key]) || deck[key].length !== 10) {
    throw new Error(`Expected ${key} to have 10 card numbers`);
  }
}

const summary = {
  selectedStartingTide: selected.startingTide,
  grantedCrystal: selected.grantedCrystal,
  questInitialDeckSize: quest.initialDeckSize,
  startingDeckTotal: deck.totalDeckSize,
  dreamcallerOffers: events
    .filter((entry) => entry.event === "dreamcaller_offers_generated")
    .flatMap((entry) => entry.offers ?? []),
  shopInventoryEvents: events.filter((entry) => entry.event === "shop_inventory_generated").length,
  rewardEvents: events.filter((entry) => entry.event === "reward_generated").length,
  finalCardAddEvents: events.filter((entry) => entry.event === "card_added").length,
};

console.log(JSON.stringify(summary, null, 2));
