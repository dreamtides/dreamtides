import { dreamcallerAccentTide } from "./quest-content";
import type { DreamcallerContent } from "../types/content";
import type { Dreamcaller } from "../types/quest";

/** Pick a stable random offer of Dreamcallers without replacement. */
export function selectDreamcallerOffer(
  dreamcallers: readonly DreamcallerContent[],
  offerSize = 3,
): DreamcallerContent[] {
  const pool = [...dreamcallers];

  for (let index = pool.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(Math.random() * (index + 1));
    [pool[index], pool[swapIndex]] = [pool[swapIndex], pool[index]];
  }

  return pool.slice(0, offerSize);
}

/** Convert normalized Dreamcaller content into the legacy quest-state shape. */
export function toSelectedDreamcaller(
  dreamcaller: DreamcallerContent,
): Dreamcaller {
  const tide = dreamcallerAccentTide(dreamcaller);

  return {
    name: dreamcaller.name,
    tide,
    abilityDescription: dreamcaller.renderedText,
    essenceBonus: 0,
    tideCrystalGrant: tide,
  };
}
