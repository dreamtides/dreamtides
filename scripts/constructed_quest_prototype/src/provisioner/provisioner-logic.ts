import type { ProvisionerOption, SiteType } from "../types/quest";

/** All purchasable site types with their costs. */
const PROVISIONER_CATALOG: ReadonlyArray<{ siteType: SiteType; cost: number }> =
  [
    { siteType: "Forge", cost: 100 },
    { siteType: "Transfiguration", cost: 75 },
    { siteType: "Duplication", cost: 75 },
    { siteType: "DraftSite", cost: 100 },
    { siteType: "DreamsignOffering", cost: 125 },
    { siteType: "LootPack", cost: 75 },
    { siteType: "Essence", cost: 50 },
  ];

/**
 * Generates 3 distinct purchasable site options from the provisioner catalog.
 * Uses Fisher-Yates partial shuffle to select without replacement.
 */
export function generateProvisionerOptions(): ProvisionerOption[] {
  const pool = [...PROVISIONER_CATALOG];
  const count = Math.min(3, pool.length);
  const result: ProvisionerOption[] = [];

  for (let i = 0; i < count; i++) {
    const j = i + Math.floor(Math.random() * (pool.length - i));
    [pool[i], pool[j]] = [pool[j], pool[i]];
    result.push({
      siteType: pool[i].siteType,
      cost: pool[i].cost,
      purchased: false,
    });
  }

  return result;
}
