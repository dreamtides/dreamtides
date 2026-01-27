import type { MenuConfig } from "@univerjs/ui";

/**
 * Menu items hidden in TV because they correspond to spreadsheet-native
 * operations that are nonfunctional in a TOML-backed data viewer.
 */
export const DISABLED_MENU_ITEMS: MenuConfig = {
  // Per-cell text formatting
  "sheet.command.set-range-bold": { hidden: true },
  "sheet.command.set-range-italic": { hidden: true },
  "sheet.command.set-range-underline": { hidden: true },
  "sheet.command.set-range-stroke": { hidden: true },
  "sheet.command.set-range-font-family": { hidden: true },
  "sheet.command.set-range-fontsize": { hidden: true },
  "sheet.command.set-range-font-increase": { hidden: true },
  "sheet.command.set-range-font-decrease": { hidden: true },
  "sheet.command.set-range-text-color": { hidden: true },
  "sheet.command.reset-range-text-color": { hidden: true },
  "sheet.command.set-range-superscript": { hidden: true },
  "sheet.command.set-range-subscript": { hidden: true },

  // Per-cell visual formatting
  "sheet.command.set-background-color": { hidden: true },
  "sheet.command.reset-background-color": { hidden: true },
  "sheet.command.set-border": { hidden: true },
  "sheet.command.set-border-basic": { hidden: true },
  "sheet.command.set-border-position": { hidden: true },
  "sheet.command.set-border-style": { hidden: true },
  "sheet.command.set-border-color": { hidden: true },
  "sheet.command.set-text-rotation": { hidden: true },

  // Cell structure operations
  "sheet.command.add-worksheet-merge": { hidden: true },
  "sheet.command.add-worksheet-merge-all": { hidden: true },
  "sheet.command.add-worksheet-merge-vertical": { hidden: true },
  "sheet.command.add-worksheet-merge-horizontal": { hidden: true },
  "sheet.command.remove-worksheet-merge": { hidden: true },
  "sheet.operation.set-format-painter": { hidden: true },
  "sheet.command.set-once-format-painter": { hidden: true },
  "sheet.command.set-infinite-format-painter": { hidden: true },
  "sheet.command.apply-format-painter": { hidden: true },
  "default-format-painter": { hidden: true },
  "sheet.command.clear-selection-format": { hidden: true },
  "sheet.command.clear-selection-all": { hidden: true },
  "sheet.menu.clear-selection": { hidden: true },

  // Row & column structure operations
  "sheet.command.insert-row-before": { hidden: true },
  "sheet.command.insert-row-after": { hidden: true },
  "sheet.command.insert-row-by-range": { hidden: true },
  "sheet.command.insert-multi-rows-above": { hidden: true },
  "sheet.command.insert-multi-rows-after": { hidden: true },
  "sheet.command.remove-row-by-range": { hidden: true },
  "sheet.command.remove-row-confirm": { hidden: true },
  "sheet.command.insert-col-before": { hidden: true },
  "sheet.command.insert-col-after": { hidden: true },
  "sheet.command.insert-col-by-range": { hidden: true },
  "sheet.command.insert-multi-cols-before": { hidden: true },
  "sheet.command.insert-multi-cols-right": { hidden: true },
  "sheet.command.remove-col-by-range": { hidden: true },
  "sheet.command.remove-col-confirm": { hidden: true },
  "sheet.menu.cell-insert": { hidden: true },
  "sheet.menu.delete": { hidden: true },

  // Sheet management
  "sheet.command.insert-sheet": { hidden: true },
  "sheet.command.remove-sheet": { hidden: true },
  "sheet.command.remove-sheet-confirm": { hidden: true },
  "sheet.command.set-worksheet-name": { hidden: true },
  "sheet.operation.rename-sheet": { hidden: true },
  "sheet.command.set-worksheet-order": { hidden: true },
  "sheet.command.set-worksheet-hidden": { hidden: true },
  "sheet.command.set-worksheet-show": { hidden: true },
  "sheet.command.set-tab-color": { hidden: true },

  // Insert objects
  "sheet.command.insert-sheet-image": { hidden: true },
  "sheet.command.insert-float-image": { hidden: true },
  "sheet.command.insert-cell-image": { hidden: true },

  // Formulas & named ranges
  "formula-ui.operation.insert-function": { hidden: true },
  "formula-ui.operation.help-function": { hidden: true },
  "formula-ui.operation.search-function": { hidden: true },
  "formula-ui.operation.more-functions": { hidden: true },
  "sheet.command.insert-defined-name": { hidden: true },
  "sheet.command.set-defined-name": { hidden: true },
  "sheet.command.remove-defined-name": { hidden: true },

  // Context menu items
  "sheet.contextMenu.permission": { hidden: true },
  "sheet.contextMenu.text-to-number": { hidden: true },
  "sheet.menu.copy-special": { hidden: true },
  "sheet.menu.paste-special": { hidden: true },
  "sheet.command.paste-format": { hidden: true },
  "sheet.command.paste-col-width": { hidden: true },
  "sheet.command.paste-besides-border": { hidden: true },
  "sheet.command.paste-formula": { hidden: true },
  "sheet.command.copy-formula-only": { hidden: true },
  "sheet.command.optional-paste": { hidden: true },

  // Protection (not applicable to TV)
  "sheet.command.add-range-protection": { hidden: true },
  "sheet.command.add-range-protection-from-toolbar": { hidden: true },
  "sheet.command.add-range-protection-from-sheet-bar": { hidden: true },
  "sheet.command.add-range-protection-from-context-menu": { hidden: true },
  "sheet.command.set-range-protection-from-context-menu": { hidden: true },
  "sheet.command.delete-range-protection": { hidden: true },
  "sheet.command.delete-range-protection-from-context-menu": { hidden: true },
  "sheet.command.view-sheet-permission-from-context-menu": { hidden: true },
  "sheet.command.view-sheet-permission-from-sheet-bar": { hidden: true },
  "sheet.command.add-worksheet-protection": { hidden: true },
  "sheet.command.set-worksheet-protection": { hidden: true },
  "sheet.command.delete-worksheet-protection": { hidden: true },
  "sheet.command.change-sheet-protection-from-sheet-bar": { hidden: true },
  "sheet.command.delete-worksheet-protection-from-sheet-bar": { hidden: true },
  "sheet.command.set-worksheet-permission-points": { hidden: true },
};
