import { useCallback, useState } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { DeckViewer } from "../components/DeckViewer";
import { logEvent } from "../logging";

/** Shows the starting deck in DeckViewer mode before transitioning to the dreamscape. */
export function StartingDeckScreen() {
  const { mutations, cardDatabase } = useQuest();
  const [viewerOpen, setViewerOpen] = useState(true);

  const handleClose = useCallback(() => {
    setViewerOpen(false);
    logEvent("starting_deck_viewed", {});
    mutations.setScreen({ type: "dreamscape" });
  }, [mutations]);

  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-4">
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.3 }}
      >
        <DeckViewer
          isOpen={viewerOpen}
          onClose={handleClose}
          cardDatabase={cardDatabase}
          initialSize="medium"
        />
      </motion.div>
    </div>
  );
}
