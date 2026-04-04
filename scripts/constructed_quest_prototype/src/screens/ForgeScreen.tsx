import { useCallback, useEffect, useMemo, useState } from "react";
import { motion } from "framer-motion";
import type { SiteState, DeckEntry } from "../types/quest";
import type { ForgeRecipe } from "../types/quest";
import type { CardData } from "../types/cards";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { logEvent } from "../logging";
import { CardDisplay } from "../components/CardDisplay";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import {
  generateForgeRecipes,
  getForgeEligibleCards,
} from "../forge/forge-logic";

/** Props for the ForgeScreen component. */
interface ForgeScreenProps {
  site: SiteState;
}

/** Forge site screen: sacrifice cards of one tide to gain a card of another. */
export function ForgeScreen({ site }: ForgeScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();
  const { pool, deck, startingTides } = state;

  const [recipes, setRecipes] = useState<ForgeRecipe[]>([]);
  const [completedRecipes, setCompletedRecipes] = useState<Set<number>>(
    new Set(),
  );
  const [pickingRecipeIndex, setPickingRecipeIndex] = useState<number | null>(
    null,
  );
  const [selectedSacrifices, setSelectedSacrifices] = useState<Set<string>>(
    new Set(),
  );
  const [pickingOutput, setPickingOutput] = useState(false);
  const [enhancedCards, setEnhancedCards] = useState<CardData[]>([]);

  // Generate recipes on mount
  useEffect(() => {
    const generated = generateForgeRecipes(
      cardDatabase,
      pool,
      deck,
      config,
      site.isEnhanced,
    );
    setRecipes(generated);
    logEvent("site_entered", {
      siteType: "Forge",
      isEnhanced: site.isEnhanced,
      recipeCount: generated.length,
    });
  }, []);

  const currentRecipe =
    pickingRecipeIndex !== null ? recipes[pickingRecipeIndex] : null;

  // Pool cards eligible for sacrifice (matching sacrifice tide, non-bane)
  const sacrificeCandidates = useMemo<DeckEntry[]>(() => {
    if (!currentRecipe) return [];
    return pool.filter((entry) => {
      if (entry.isBane) return false;
      const card = cardDatabase.get(entry.cardNumber);
      return card !== undefined && card.tide === currentRecipe.sacrificeTide;
    });
  }, [pool, cardDatabase, currentRecipe]);

  const requiredCount = currentRecipe?.sacrificeCount ?? config.forgeCost;

  const handleStartForge = useCallback((index: number) => {
    setPickingRecipeIndex(index);
    setSelectedSacrifices(new Set());
    setPickingOutput(false);
  }, []);

  const handleToggleSacrifice = useCallback(
    (entryId: string) => {
      setSelectedSacrifices((prev) => {
        const next = new Set(prev);
        if (next.has(entryId)) {
          next.delete(entryId);
        } else if (next.size < requiredCount) {
          next.add(entryId);
        }
        return next;
      });
    },
    [requiredCount],
  );

  const handleConfirmSacrifice = useCallback(() => {
    if (!currentRecipe || pickingRecipeIndex === null) return;

    if (site.isEnhanced && currentRecipe.outputCard === null) {
      // Enhanced mode: show output picker
      const eligible = getForgeEligibleCards(
        cardDatabase,
        startingTides,
        currentRecipe.sacrificeTide,
      );
      setEnhancedCards(eligible);
      setPickingOutput(true);
      return;
    }

    // Standard mode: execute the forge
    executeForge(currentRecipe.outputCard);
  }, [currentRecipe, pickingRecipeIndex, site.isEnhanced, cardDatabase, startingTides]);

  const executeForge = useCallback(
    (outputCard: CardData | null) => {
      if (!currentRecipe || pickingRecipeIndex === null || !outputCard) return;

      // Remove sacrificed cards
      for (const entryId of selectedSacrifices) {
        mutations.removeFromPool(entryId, "forge_sacrifice");
      }

      // Add output card
      mutations.addToPool(outputCard.cardNumber, "forge_output");

      logEvent("forge_completed", {
        sacrificeTide: currentRecipe.sacrificeTide,
        sacrificeCount: selectedSacrifices.size,
        outputCardNumber: outputCard.cardNumber,
        outputCardName: outputCard.name,
        isEnhanced: site.isEnhanced,
      });

      setCompletedRecipes((prev) => new Set([...prev, pickingRecipeIndex]));
      setPickingRecipeIndex(null);
      setSelectedSacrifices(new Set());
      setPickingOutput(false);
      setEnhancedCards([]);
    },
    [currentRecipe, pickingRecipeIndex, selectedSacrifices, mutations, site.isEnhanced],
  );

  const handleSelectEnhancedCard = useCallback(
    (card: CardData) => {
      executeForge(card);
    },
    [executeForge],
  );

  const handleCancelPick = useCallback(() => {
    setPickingRecipeIndex(null);
    setSelectedSacrifices(new Set());
    setPickingOutput(false);
    setEnhancedCards([]);
  }, []);

  const handleDone = useCallback(() => {
    logEvent("site_completed", {
      siteType: "Forge",
      isEnhanced: site.isEnhanced,
      recipesCompleted: completedRecipes.size,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations, completedRecipes.size]);

  // Enhanced output picker
  if (pickingOutput) {
    return (
      <motion.div
        className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4 }}
      >
        <div className="mb-6 text-center">
          <h2
            className="text-2xl font-bold tracking-wide md:text-3xl"
            style={{ color: "#f59e0b" }}
          >
            Choose Your Reward
          </h2>
          <p className="mt-1 text-sm opacity-50">
            Select a card to gain from the forge
          </p>
        </div>

        <div className="mb-8 flex flex-wrap justify-center gap-4">
          {enhancedCards.map((card) => (
            <motion.div
              key={card.cardNumber}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.3 }}
              style={{ width: "160px" }}
            >
              <CardDisplay
                card={card}
                onClick={() => handleSelectEnhancedCard(card)}
              />
            </motion.div>
          ))}
        </div>

        <button
          className="rounded-lg px-6 py-2.5 font-medium transition-colors"
          style={{
            background: "rgba(107, 114, 128, 0.2)",
            border: "1px solid rgba(107, 114, 128, 0.4)",
            color: "#9ca3af",
          }}
          onClick={handleCancelPick}
        >
          Cancel
        </button>
      </motion.div>
    );
  }

  // Card sacrifice picker
  if (pickingRecipeIndex !== null && currentRecipe) {
    const tideColor = TIDE_COLORS[currentRecipe.sacrificeTide];

    return (
      <motion.div
        className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4 }}
      >
        <div className="mb-6 text-center">
          <h2
            className="text-2xl font-bold tracking-wide md:text-3xl"
            style={{ color: tideColor }}
          >
            Select Cards to Sacrifice
          </h2>
          <p className="mt-1 text-sm opacity-50">
            Choose {String(requiredCount)} {currentRecipe.sacrificeTide} cards ({String(selectedSacrifices.size)}/{String(requiredCount)} selected)
          </p>
        </div>

        <div className="mb-6 flex flex-wrap justify-center gap-3">
          {sacrificeCandidates.map((entry) => {
            const card = cardDatabase.get(entry.cardNumber);
            if (!card) return null;
            const isSelected = selectedSacrifices.has(entry.entryId);
            return (
              <motion.div
                key={entry.entryId}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.3 }}
                style={{ width: "150px" }}
              >
                <CardDisplay
                  card={card}
                  onClick={() => handleToggleSacrifice(entry.entryId)}
                  selected={isSelected}
                  selectionColor="#dc2626"
                />
              </motion.div>
            );
          })}
        </div>

        <div className="flex gap-4">
          <motion.button
            className="rounded-lg px-8 py-3 text-lg font-bold text-white transition-opacity disabled:opacity-40"
            style={{
              background:
                selectedSacrifices.size === requiredCount
                  ? "linear-gradient(135deg, #f59e0b 0%, #d97706 100%)"
                  : "rgba(107, 114, 128, 0.3)",
              boxShadow:
                selectedSacrifices.size === requiredCount
                  ? "0 0 20px rgba(245, 158, 11, 0.3)"
                  : "none",
            }}
            whileHover={
              selectedSacrifices.size === requiredCount
                ? { scale: 1.05 }
                : undefined
            }
            whileTap={
              selectedSacrifices.size === requiredCount
                ? { scale: 0.97 }
                : undefined
            }
            disabled={selectedSacrifices.size !== requiredCount}
            onClick={handleConfirmSacrifice}
          >
            Sacrifice
          </motion.button>
          <button
            className="rounded-lg px-6 py-3 font-medium transition-colors"
            style={{
              background: "rgba(107, 114, 128, 0.2)",
              border: "1px solid rgba(107, 114, 128, 0.4)",
              color: "#9ca3af",
            }}
            onClick={handleCancelPick}
          >
            Cancel
          </button>
        </div>
      </motion.div>
    );
  }

  // Main recipe display
  return (
    <motion.div
      className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.4 }}
    >
      {/* Header */}
      <div className="mb-6 text-center">
        <h2
          className="text-2xl font-bold tracking-wide md:text-3xl"
          style={{ color: "#f59e0b" }}
        >
          {site.isEnhanced ? "Shadowforge" : "Forge"}
        </h2>
        {site.isEnhanced && (
          <span
            className="mt-2 inline-block rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(245, 158, 11, 0.15)",
              color: "#fbbf24",
              border: "1px solid rgba(245, 158, 11, 0.3)",
            }}
          >
            Enhanced
          </span>
        )}
        <p className="mt-2 text-sm opacity-50">
          Sacrifice {String(config.forgeCost)} cards of one tide to gain a card of
          another
        </p>
      </div>

      {/* Recipes */}
      {recipes.length === 0 ? (
        <div className="mb-8 text-center">
          <p className="text-lg opacity-50">
            No forge recipes available. You need at least{" "}
            {String(config.forgeCost)} cards of a single tide to forge.
          </p>
        </div>
      ) : (
        <div className="mb-8 flex flex-wrap justify-center gap-6">
          {recipes.map((recipe, index) => {
            const isCompleted = completedRecipes.has(index);
            return (
              <RecipeCard
                key={`${recipe.sacrificeTide}-${String(index)}`}
                recipe={recipe}
                index={index}
                isCompleted={isCompleted}
                isEnhanced={site.isEnhanced}
                onForge={handleStartForge}
              />
            );
          })}
        </div>
      )}

      {/* Bottom actions */}
      <div className="flex gap-4">
        <button
          className="rounded-lg px-8 py-3 text-lg font-medium transition-colors"
          style={{
            background:
              completedRecipes.size > 0
                ? "linear-gradient(135deg, #7c3aed 0%, #5b21b6 100%)"
                : "rgba(107, 114, 128, 0.2)",
            border:
              completedRecipes.size > 0
                ? "none"
                : "1px solid rgba(107, 114, 128, 0.4)",
            color: completedRecipes.size > 0 ? "#ffffff" : "#9ca3af",
          }}
          onClick={handleDone}
        >
          {completedRecipes.size > 0 ? "Done" : "Decline All"}
        </button>
      </div>
    </motion.div>
  );
}

/** Displays a single forge recipe with sacrifice/output info. */
function RecipeCard({
  recipe,
  index,
  isCompleted,
  isEnhanced,
  onForge,
}: {
  recipe: ForgeRecipe;
  index: number;
  isCompleted: boolean;
  isEnhanced: boolean;
  onForge: (index: number) => void;
}) {
  const tideColor = TIDE_COLORS[recipe.sacrificeTide];

  return (
    <motion.div
      className="flex w-72 flex-col items-center gap-4 rounded-xl p-5"
      style={{
        background: isCompleted
          ? "linear-gradient(145deg, #0a1a0a 0%, #0a180a 60%, #0d0814 100%)"
          : "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: isCompleted
          ? "1px solid rgba(16, 185, 129, 0.3)"
          : `1px solid ${tideColor}40`,
        opacity: isCompleted ? 0.6 : 1,
      }}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: isCompleted ? 0.6 : 1, y: 0 }}
      transition={{ delay: index * 0.1, duration: 0.4 }}
    >
      {/* Sacrifice info */}
      <div className="flex items-center gap-2">
        <img
          src={tideIconUrl(recipe.sacrificeTide)}
          alt={recipe.sacrificeTide}
          className="h-8 w-8 rounded-full object-contain"
          style={{ border: `2px solid ${tideColor}` }}
        />
        <span className="text-sm font-bold" style={{ color: tideColor }}>
          Sacrifice {String(recipe.sacrificeCount)} {recipe.sacrificeTide}
        </span>
      </div>

      {/* Arrow */}
      <span className="text-2xl opacity-40" style={{ color: "#f59e0b" }}>
        {"\u2193"}
      </span>

      {/* Output */}
      {isEnhanced || recipe.outputCard === null ? (
        <div className="flex flex-col items-center gap-1">
          <span className="text-sm font-bold" style={{ color: "#fbbf24" }}>
            Choose a card
          </span>
          <span className="text-xs opacity-50">
            Pick from your starting tides
          </span>
        </div>
      ) : (
        <div style={{ width: "160px" }}>
          <CardDisplay card={recipe.outputCard} />
        </div>
      )}

      {/* Forge button */}
      {isCompleted ? (
        <span
          className="rounded-full px-4 py-1.5 text-sm font-bold"
          style={{ color: "#10b981" }}
        >
          Forged
        </span>
      ) : (
        <motion.button
          className="rounded-lg px-6 py-2.5 font-bold text-white"
          style={{
            background: "linear-gradient(135deg, #f59e0b 0%, #d97706 100%)",
            boxShadow: "0 0 15px rgba(245, 158, 11, 0.25)",
          }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
          onClick={() => onForge(index)}
        >
          Forge
        </motion.button>
      )}
    </motion.div>
  );
}
