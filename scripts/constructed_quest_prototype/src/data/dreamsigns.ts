import type { Dreamsign } from "../types/quest";

/** Static dreamsign definitions without bane status (set when acquired). */
type DreamsignTemplate = Omit<Dreamsign, "isBane">;

/** The 10 available dreamsigns, distributed across all 8 tides. */
export const DREAMSIGNS: readonly DreamsignTemplate[] = [
  {
    name: "Ember's Whisper",
    tide: "Ignite",
    effectDescription:
      "Your characters burn with inner fire, dealing lingering damage that persists between turns.",
  },
  {
    name: "Glacial Insight",
    tide: "Rime",
    effectDescription:
      "You foresee one additional card at the start of each battle, gaining clarity through stillness.",
  },
  {
    name: "Verdant Accord",
    tide: "Bloom",
    effectDescription:
      "Essence spent on characters returns a small portion as healing, roots sustaining what you nurture.",
  },
  {
    name: "Stormthread Sigil",
    tide: "Arc",
    effectDescription:
      "Shops offer an additional item, as merchants sense the electric potential you carry.",
  },
  {
    name: "Hollowed Reflection",
    tide: "Umbra",
    effectDescription:
      "Dream journeys reveal a third option, drawn from the shadow of what might have been.",
  },
  {
    name: "Crimson Covenant",
    tide: "Pact",
    effectDescription:
      "Tempting offers cost 20% less essence, as the dream brokers recognize a kindred spirit.",
  },
  {
    name: "Abyssal Resonance",
    tide: "Surge",
    effectDescription:
      "Purging a card from your deck grants a small essence refund, the deep reclaiming what is cast away.",
  },
  {
    name: "Untamed Compass",
    tide: "Neutral",
    effectDescription:
      "New dreamscapes generate one additional site, the wild current carving unexpected paths.",
  },
  {
    name: "Flickering Ward",
    tide: "Rime",
    effectDescription:
      "Your first event each battle cannot be prevented, sealed behind a shell of ancient ice.",
  },
  {
    name: "Ashbloom Mantle",
    tide: "Bloom",
    effectDescription:
      "Characters you materialize gain a spark of vitality, life blooming even in scorched ground.",
  },
] as const;
