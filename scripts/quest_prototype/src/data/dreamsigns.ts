import type { DreamsignTemplate } from "../types/content";
import type { Dreamsign } from "../types/quest";

/** The 10 available Dreamsign templates with hidden package-tide membership. */
export const DREAMSIGN_TEMPLATES: readonly DreamsignTemplate[] = [
  {
    id: "embers-whisper",
    name: "Ember's Whisper",
    displayTide: "Ignite",
    packageTides: ["event_chain", "fast_tempo", "point_pressure"],
    effectDescription:
      "Your characters burn with inner fire, dealing lingering damage that persists between turns.",
  },
  {
    id: "glacial-insight",
    name: "Glacial Insight",
    displayTide: "Rime",
    packageTides: ["foresee_selection", "judgment_engines", "topdeck_setup"],
    effectDescription:
      "You foresee one additional card at the start of each battle, gaining clarity through stillness.",
  },
  {
    id: "verdant-accord",
    name: "Verdant Accord",
    displayTide: "Bloom",
    packageTides: ["spirit_growth", "defensive_curve", "materialized_staples"],
    effectDescription:
      "Essence spent on characters returns a small portion as healing, roots sustaining what you nurture.",
  },
  {
    id: "stormthread-sigil",
    name: "Stormthread Sigil",
    displayTide: "Arc",
    packageTides: ["resource_burst", "discover_toolbox", "big_energy"],
    effectDescription:
      "Shops offer an additional item, as merchants sense the electric potential you carry.",
  },
  {
    id: "hollowed-reflection",
    name: "Hollowed Reflection",
    displayTide: "Umbra",
    packageTides: ["void_setup", "discover_toolbox", "foresee_selection"],
    effectDescription:
      "Dream journeys reveal a third option, drawn from the shadow of what might have been.",
  },
  {
    id: "crimson-covenant",
    name: "Crimson Covenant",
    displayTide: "Pact",
    packageTides: ["cost_reduction", "hand_disruption", "big_energy"],
    effectDescription:
      "Tempting offers cost 20% less essence, as the dream brokers recognize a kindred spirit.",
  },
  {
    id: "abyssal-resonance",
    name: "Abyssal Resonance",
    displayTide: "Surge",
    packageTides: ["void_recursion", "reclaim_characters", "leave_play_enablers"],
    effectDescription:
      "Purging a card from your deck grants a small essence refund, the deep reclaiming what is cast away.",
  },
  {
    id: "untamed-compass",
    name: "Untamed Compass",
    displayTide: "Neutral",
    packageTides: ["discover_toolbox", "card_flow", "midcurve_glue"],
    effectDescription:
      "New dreamscapes generate one additional site, the wild current carving unexpected paths.",
  },
  {
    id: "flickering-ward",
    name: "Flickering Ward",
    displayTide: "Rime",
    packageTides: ["prevent_control", "fast_interaction", "fast_setup"],
    effectDescription:
      "Your first event each battle cannot be prevented, sealed behind a shell of ancient ice.",
  },
  {
    id: "ashbloom-mantle",
    name: "Ashbloom Mantle",
    displayTide: "Bloom",
    packageTides: ["materialize_value", "spirit_growth", "trigger_reuse"],
    effectDescription:
      "Characters you materialize gain a spark of vitality, life blooming even in scorched ground.",
  },
] as const;

/** Instantiates a collectible Dreamsign from a template. */
export function createDreamsign(
  template: DreamsignTemplate,
  isBane = false,
): Dreamsign {
  return {
    name: template.name,
    tide: template.displayTide,
    effectDescription: template.effectDescription,
    isBane,
  };
}

/** Compatibility collection for legacy screens that still expect instantiated signs. */
export const DREAMSIGNS: readonly Omit<Dreamsign, "isBane">[] =
  DREAMSIGN_TEMPLATES.map((template) => createDreamsign(template));
