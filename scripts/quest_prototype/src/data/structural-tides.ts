import type { PackageTideId } from "../types/content";

export interface StructuralTideMeta {
  displayName: string;
  hoverBlurb: string;
  iconClass: string;
}

export const STRUCTURAL_TIDE_META: Readonly<
  Partial<Record<PackageTideId, StructuralTideMeta>>
> = {
  warrior_pressure: {
    displayName: "Iron Charge",
    hoverBlurb:
      "A war drum beat made into doctrine. The first bodies hit hard, then every follow-up turns the field into a sprint the enemy cannot survive.",
    iconClass: "bx-bullseye",
  },
  warrior_bastion: {
    displayName: "Stone Rampart",
    hoverBlurb:
      "The line does not break. Shields lock, trades stay favorable, and every failed assault leaves the other side bloodier than before.",
    iconClass: "bxs-castle",
  },
  spirit_growth: {
    displayName: "Verdant Ascent",
    hoverBlurb:
      "Life gathers momentum in secret roots. Small turns become rich turns, rich turns become overwhelming ones, and the dream keeps flowering upward.",
    iconClass: "bxs-tree-alt",
  },
  materialize_value: {
    displayName: "Echo Arrival",
    hoverBlurb:
      "Arrival is never just arrival. Every entrance leaves behind an extra page of value, until replay itself feels like drawing breath.",
    iconClass: "bx-book",
  },
  materialize_tempo: {
    displayName: "Flicker Rush",
    hoverBlurb:
      "Nothing stays pinned down for long. Allies slip sideways through reality, blockers vanish for a heartbeat, and that heartbeat is enough to win.",
    iconClass: "bx-show",
  },
  ally_formation: {
    displayName: "Banner Formation",
    hoverBlurb:
      "This tide fights like a drilled company. Position matters, timing matters, and a disciplined line turns ordinary allies into a precise machine.",
    iconClass: "bx-diamond",
  },
  ally_wide: {
    displayName: "Rising Host",
    hoverBlurb:
      "A single threat can be answered. A battlefield that keeps filling cannot. The host grows until the whole dream is occupied.",
    iconClass: "bx-sun",
  },
  fast_tempo: {
    displayName: "Quickened Edge",
    hoverBlurb:
      "Victory lives in the half-second before the rival is ready. This tide steals initiative, acts at impossible moments, and never gives it back.",
    iconClass: "bxs-bolt",
  },
  event_chain: {
    displayName: "Arc Cascade",
    hoverBlurb:
      "One spell cracks the air open for the next. The turn keeps surging, costs fall away, and the chain ends only when something decisive breaks.",
    iconClass: "bx-meteor",
  },
  prevent_control: {
    displayName: "Sealed Verdict",
    hoverBlurb:
      "The enemy reaches for a plan and finds a closed door. This tide wins by refusal, until denial itself becomes a form of pressure.",
    iconClass: "bx-shield-x",
  },
  discard_velocity: {
    displayName: "Cinder Sprint",
    hoverBlurb:
      "The hand burns hot and fast. What gets thrown away becomes fuel, and the deck races forward on sparks from its own losses.",
    iconClass: "bxs-flame",
  },
  void_recursion: {
    displayName: "Haunting Return",
    hoverBlurb:
      "Nothing properly leaves. The void keeps its own ledger, and what was spent comes stalking back when the moment is right.",
    iconClass: "bxs-ghost",
  },
  void_threshold: {
    displayName: "Moonlit Threshold",
    hoverBlurb:
      "Power waits below the surface count. Once the void is deep enough, the tide crosses a line and every payoff comes back colder and larger.",
    iconClass: "bxs-moon",
  },
  abandon_value: {
    displayName: "Grave Bargain",
    hoverBlurb:
      "Every sacrifice is a transaction. Bodies are spent for cards, energy, and leverage until loss itself starts looking profitable.",
    iconClass: "bxs-skull",
  },
  abandon_ladder: {
    displayName: "Funeral Ladder",
    hoverBlurb:
      "Smaller pieces are not mourned, they are climbed. Each offered body becomes the rung that lifts the next threat into the world.",
    iconClass: "bx-pyramid",
  },
  figment_swarm: {
    displayName: "Dream Shards",
    hoverBlurb:
      "Fragments breed fragments. The board fills with glittering unreal bodies until the dream stops feeling imagined and starts feeling inevitable.",
    iconClass: "bxs-star-half",
  },
  survivor_dissolve: {
    displayName: "Enduring Wake",
    hoverBlurb:
      "Death is only another phase of the march. Survivors fall, leave something useful behind, and return often enough to make attrition futile.",
    iconClass: "bxs-yin-yang",
  },
  judgment_engines: {
    displayName: "Turning Hourglass",
    hoverBlurb:
      "This tide wins by tampering with the hour of reckoning. Judgment arrives heavier, repeats itself, and turns the scoring phase into a weapon.",
    iconClass: "bx-hourglass",
  },
  character_chain: {
    displayName: "Living Procession",
    hoverBlurb:
      "Each body invites the next. The turn becomes a procession of arrivals, rebates, and chained deployments that never quite stop on schedule.",
    iconClass: "bx-cloud-lightning",
  },
  spark_tall: {
    displayName: "Kindled Crown",
    hoverBlurb:
      "All strength is gathered into a chosen few. One threat grows radiant enough to rule the board while lesser bodies exist only to feed it.",
    iconClass: "bx-crown",
  },
} as const;

export function structuralTidesForPackageTides(
  packageTides: readonly PackageTideId[],
): Array<StructuralTideMeta & { id: PackageTideId }> {
  return packageTides.flatMap((packageTideId) => {
    const structuralTide = STRUCTURAL_TIDE_META[packageTideId];
    return structuralTide === undefined
      ? []
      : [{ id: packageTideId, ...structuralTide }];
  });
}
