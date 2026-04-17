import type { DreamsignTemplate } from "../types/content";
import type { Dreamsign } from "../types/quest";

/** Instantiates a collectible Dreamsign from canonical template data. */
export function createDreamsign(
  template: DreamsignTemplate,
  isBane = false,
): Dreamsign {
  return {
    id: template.id,
    name: template.name,
    effectDescription: template.effectDescription,
    imageName: template.imageName,
    imageAlt: template.imageAlt ?? `${template.name} Dreamsign artwork`,
    tide: null,
    isBane,
  };
}
