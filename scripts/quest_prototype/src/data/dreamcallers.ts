import type { Dreamcaller } from "../types/quest";

/** The 25 available dreamcallers, distributed across all 8 tides. */
export const DREAMCALLERS: readonly Dreamcaller[] = [
  // --- Bloom (3) ---
  {
    name: "Lyria, Tide Weaver",
    tide: "Bloom",
    abilityDescription:
      "She draws vitality from the roots of dreaming trees, mending wounds with whispered verse.",
    essenceBonus: 80,
    tideCrystalGrant: "Bloom",
  },
  {
    name: "Thorn of the Green Veil",
    tide: "Bloom",
    abilityDescription:
      "Where he walks, forgotten gardens rise again. Even barren stone cracks open to release sleeping buds.",
    essenceBonus: 60,
    tideCrystalGrant: "Bloom",
  },
  {
    name: "Caelindra, Verdant Seer",
    tide: "Bloom",
    abilityDescription:
      "She reads the seasons of dreams yet to come, planting seeds in futures that have not yet unfolded.",
    essenceBonus: 110,
    tideCrystalGrant: "Bloom",
  },

  // --- Arc (3) ---
  {
    name: "Kael of the Ashen Veil",
    tide: "Arc",
    abilityDescription:
      "Lightning traces his every step. He reads the storm-patterns that pulse between realms.",
    essenceBonus: 100,
    tideCrystalGrant: "Arc",
  },
  {
    name: "Thessra, Bolt Maiden",
    tide: "Arc",
    abilityDescription:
      "She catches lightning in glass jars and releases it as whispered truths that shatter certainty.",
    essenceBonus: 70,
    tideCrystalGrant: "Arc",
  },
  {
    name: "Runos the Charged",
    tide: "Arc",
    abilityDescription:
      "His laughter crackles with static. He bends the arc of fate like a drawn bow, aiming for impossibility.",
    essenceBonus: 130,
    tideCrystalGrant: "Arc",
  },

  // --- Ignite (3) ---
  {
    name: "Serath, the Cindermaw",
    tide: "Ignite",
    abilityDescription:
      "Flame dances in her eyes and boils the air around her. She burns away all that is false.",
    essenceBonus: 60,
    tideCrystalGrant: "Ignite",
  },
  {
    name: "Vaelith, Ember Augur",
    tide: "Ignite",
    abilityDescription:
      "He reads the future in ashes and cinder. Every flame he kindles reveals a hidden truth.",
    essenceBonus: 50,
    tideCrystalGrant: "Ignite",
  },
  {
    name: "Pyrra, the Molten Heart",
    tide: "Ignite",
    abilityDescription:
      "Her blood is liquid fire, and every heartbeat sends warmth through frozen dreamscapes.",
    essenceBonus: 90,
    tideCrystalGrant: "Ignite",
  },

  // --- Pact (3) ---
  {
    name: "Mireille Duskpact",
    tide: "Pact",
    abilityDescription:
      "Her bargains are sealed in blood and starlight. Every alliance she forges bends fate itself.",
    essenceBonus: 120,
    tideCrystalGrant: "Pact",
  },
  {
    name: "Vel'thane, the Bound",
    tide: "Pact",
    abilityDescription:
      "He wears the chains of a thousand sworn oaths. Each link grants power; each link demands a price.",
    essenceBonus: 80,
    tideCrystalGrant: "Pact",
  },
  {
    name: "Sable, the Oath Keeper",
    tide: "Pact",
    abilityDescription:
      "She never forgets a promise. The dreamscape itself holds debtors accountable when she is near.",
    essenceBonus: 140,
    tideCrystalGrant: "Pact",
  },

  // --- Umbra (3) ---
  {
    name: "Thalvor the Hollow King",
    tide: "Umbra",
    abilityDescription:
      "He wears a crown of shadows and speaks to the nothing between worlds. Even light obeys him.",
    essenceBonus: 70,
    tideCrystalGrant: "Umbra",
  },
  {
    name: "Orivane, the Pale Witness",
    tide: "Umbra",
    abilityDescription:
      "She has seen the end of all dreams and returned unchanged. Her gaze unravels deception.",
    essenceBonus: 130,
    tideCrystalGrant: "Umbra",
  },
  {
    name: "Noctis, the Fading",
    tide: "Umbra",
    abilityDescription:
      "He exists at the edge of perception. Things forgotten by the waking world gather in his cloak.",
    essenceBonus: 100,
    tideCrystalGrant: "Umbra",
  },

  // --- Rime (3) ---
  {
    name: "Isolde Frostborne",
    tide: "Rime",
    abilityDescription:
      "Ice crystallizes in her wake, preserving memories frozen in perfect clarity.",
    essenceBonus: 90,
    tideCrystalGrant: "Rime",
  },
  {
    name: "Glassen, the Still Mirror",
    tide: "Rime",
    abilityDescription:
      "He stands so still that time itself pauses around him, and in the silence, truths become visible.",
    essenceBonus: 60,
    tideCrystalGrant: "Rime",
  },
  {
    name: "Eira, Winter's Breath",
    tide: "Rime",
    abilityDescription:
      "Her exhaled frost carries the patience of glaciers. She waits for the perfect moment, then shatters it.",
    essenceBonus: 120,
    tideCrystalGrant: "Rime",
  },

  // --- Surge (3) ---
  {
    name: "Nyvex, Depth Strider",
    tide: "Surge",
    abilityDescription:
      "He walks the crushing deep where drowned gods slumber, carrying their forgotten songs.",
    essenceBonus: 110,
    tideCrystalGrant: "Surge",
  },
  {
    name: "Maressa, Tidal Voice",
    tide: "Surge",
    abilityDescription:
      "She sings and the currents rearrange themselves. Even whirlpools bow to her melody.",
    essenceBonus: 80,
    tideCrystalGrant: "Surge",
  },
  {
    name: "Korrath, the Undertow",
    tide: "Surge",
    abilityDescription:
      "He pulls secrets from the depths that others have cast away. Nothing discarded escapes his current.",
    essenceBonus: 50,
    tideCrystalGrant: "Surge",
  },

  // --- Neutral (4) ---
  {
    name: "Eryndra Wildsong",
    tide: "Neutral",
    abilityDescription:
      "She belongs to no tide and all tides. The raw chaos of the dreamscape answers her call.",
    essenceBonus: 150,
    tideCrystalGrant: "Neutral",
  },
  {
    name: "Wanderer Zael",
    tide: "Neutral",
    abilityDescription:
      "He has walked every tide's domain and carries a fragment of each. His loyalty is to the journey alone.",
    essenceBonus: 100,
    tideCrystalGrant: "Neutral",
  },
  {
    name: "The Dreaming Compass",
    tide: "Neutral",
    abilityDescription:
      "Not a person but a living artifact—a spinning needle of light that points toward possibility.",
    essenceBonus: 70,
    tideCrystalGrant: "Neutral",
  },
  {
    name: "Ashen, the Unaligned",
    tide: "Neutral",
    abilityDescription:
      "They shed their tide like old skin and exist in the spaces between. All paths are open to the unbound.",
    essenceBonus: 120,
    tideCrystalGrant: "Neutral",
  },
] as const;
