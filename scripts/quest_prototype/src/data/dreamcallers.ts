import type { Dreamcaller } from "../types/quest";

/** The 10 available dreamcallers, each spanning two neighboring named tides. */
export const DREAMCALLERS: readonly Dreamcaller[] = [
  {
    name: "Lyria, Tide Weaver",
    tides: ["Bloom", "Arc"],
    abilityDescription:
      "She draws vitality from the roots of dreaming trees, mending wounds with whispered verse.",
    essenceBonus: 80,
    tideCrystalGrant: "Bloom",
  },
  {
    name: "Kael of the Ashen Veil",
    tides: ["Arc", "Ignite"],
    abilityDescription:
      "Lightning traces his every step. He reads the storm-patterns that pulse between realms.",
    essenceBonus: 100,
    tideCrystalGrant: "Arc",
  },
  {
    name: "Serath, the Cindermaw",
    tides: ["Ignite", "Pact"],
    abilityDescription:
      "Flame dances in her eyes and boils the air around her. She burns away all that is false.",
    essenceBonus: 60,
    tideCrystalGrant: "Ignite",
  },
  {
    name: "Mireille Duskpact",
    tides: ["Pact", "Umbra"],
    abilityDescription:
      "Her bargains are sealed in blood and starlight. Every alliance she forges bends fate itself.",
    essenceBonus: 120,
    tideCrystalGrant: "Pact",
  },
  {
    name: "Thalvor the Hollow King",
    tides: ["Umbra", "Rime"],
    abilityDescription:
      "He wears a crown of shadows and speaks to the nothing between worlds. Even light obeys him.",
    essenceBonus: 70,
    tideCrystalGrant: "Umbra",
  },
  {
    name: "Isolde Frostborne",
    tides: ["Rime", "Surge"],
    abilityDescription:
      "Ice crystallizes in her wake, preserving memories frozen in perfect clarity.",
    essenceBonus: 90,
    tideCrystalGrant: "Rime",
  },
  {
    name: "Nyvex, Depth Strider",
    tides: ["Surge", "Bloom"],
    abilityDescription:
      "He walks the crushing deep where drowned gods slumber, carrying their forgotten songs.",
    essenceBonus: 110,
    tideCrystalGrant: "Surge",
  },
  {
    name: "Eryndra Wildsong",
    tides: ["Surge", "Bloom"],
    abilityDescription:
      "She belongs to no tide and all tides. The raw chaos of the dreamscape answers her call.",
    essenceBonus: 150,
    tideCrystalGrant: "Bloom",
  },
  {
    name: "Vaelith, Ember Augur",
    tides: ["Arc", "Ignite"],
    abilityDescription:
      "He reads the future in ashes and cinder. Every flame he kindles reveals a hidden truth.",
    essenceBonus: 50,
    tideCrystalGrant: "Ignite",
  },
  {
    name: "Orivane, the Pale Witness",
    tides: ["Pact", "Umbra"],
    abilityDescription:
      "She has seen the end of all dreams and returned unchanged. Her gaze unravels deception.",
    essenceBonus: 130,
    tideCrystalGrant: "Umbra",
  },
] as const;
