import { LocaleType, mergeLocales, Univer } from "@univerjs/core";
import { FUniver } from "@univerjs/core/facade";
import { UniverDataValidationPlugin } from "@univerjs/data-validation";
import DesignEnUS from "@univerjs/design/locale/en-US";
import { UniverDocsPlugin } from "@univerjs/docs";
import { UniverDocsUIPlugin } from "@univerjs/docs-ui";
import DocsUIEnUS from "@univerjs/docs-ui/locale/en-US";
import { UniverFormulaEnginePlugin } from "@univerjs/engine-formula";
import { UniverRenderEnginePlugin } from "@univerjs/engine-render";
import { UniverSheetsPlugin } from "@univerjs/sheets";
import { UniverSheetsConditionalFormattingPlugin } from "@univerjs/sheets-conditional-formatting";
import { UniverSheetsConditionalFormattingUIPlugin } from "@univerjs/sheets-conditional-formatting-ui";
import SheetsConditionalFormattingUIEnUS from "@univerjs/sheets-conditional-formatting-ui/locale/en-US";
import { UniverSheetsDataValidationPlugin } from "@univerjs/sheets-data-validation";
import { UniverSheetsDataValidationUIPlugin } from "@univerjs/sheets-data-validation-ui";
import SheetsDataValidationUIEnUS from "@univerjs/sheets-data-validation-ui/locale/en-US";
import { UniverSheetsFilterPlugin } from "@univerjs/sheets-filter";
import { UniverSheetsFilterUIPlugin } from "@univerjs/sheets-filter-ui";
import SheetsFilterUIEnUS from "@univerjs/sheets-filter-ui/locale/en-US";
import { UniverSheetsFormulaPlugin } from "@univerjs/sheets-formula";
import { UniverSheetsFormulaUIPlugin } from "@univerjs/sheets-formula-ui";
import SheetsFormulaUIEnUS from "@univerjs/sheets-formula-ui/locale/en-US";
import { UniverSheetsNumfmtPlugin } from "@univerjs/sheets-numfmt";
import { UniverSheetsNumfmtUIPlugin } from "@univerjs/sheets-numfmt-ui";
import SheetsNumfmtUIEnUS from "@univerjs/sheets-numfmt-ui/locale/en-US";
import { UniverSheetsSortPlugin } from "@univerjs/sheets-sort";
import { UniverSheetsSortUIPlugin } from "@univerjs/sheets-sort-ui";
import SheetsSortUIEnUS from "@univerjs/sheets-sort-ui/locale/en-US";
import { UniverSheetsUIPlugin } from "@univerjs/sheets-ui";
import SheetsUIEnUS from "@univerjs/sheets-ui/locale/en-US";
import SheetsEnUS from "@univerjs/sheets/locale/en-US";
import { UniverUIPlugin } from "@univerjs/ui";
import UIEnUS from "@univerjs/ui/locale/en-US";
import { UniverDrawingPlugin } from "@univerjs/drawing";
import { UniverDrawingUIPlugin } from "@univerjs/drawing-ui";
import DrawingUIEnUS from "@univerjs/drawing-ui/locale/en-US";
import { UniverSheetsDrawingPlugin } from "@univerjs/sheets-drawing";
import { UniverSheetsDrawingUIPlugin } from "@univerjs/sheets-drawing-ui";
import SheetsDrawingUIEnUS from "@univerjs/sheets-drawing-ui/locale/en-US";

import "@univerjs/design/lib/index.css";
import "@univerjs/ui/lib/index.css";
import "@univerjs/docs-ui/lib/index.css";
import "@univerjs/sheets-ui/lib/index.css";
import "@univerjs/sheets-formula-ui/lib/index.css";
import "@univerjs/sheets-numfmt-ui/lib/index.css";
import "@univerjs/sheets-filter-ui/lib/index.css";
import "@univerjs/sheets-conditional-formatting-ui/lib/index.css";
import "@univerjs/sheets-data-validation-ui/lib/index.css";
import "@univerjs/sheets-sort-ui/lib/index.css";
import "@univerjs/drawing-ui/lib/index.css";
import "@univerjs/sheets-drawing-ui/lib/index.css";

// Facade side-effect imports add methods to FWorkbook/FWorksheet prototypes
// via FBase.extend(). Vite pre-bundling can break this by creating duplicate
// class prototypes — the extend() runs on one copy while runtime uses another.
// See image_cell_renderer.ts for the workaround and appendix_d for details.
import "@univerjs/engine-formula/facade";
import "@univerjs/ui/facade";
import "@univerjs/docs-ui/facade";
import "@univerjs/sheets/facade";
import "@univerjs/sheets-ui/facade";
import "@univerjs/sheets-formula/facade";
import "@univerjs/sheets-numfmt/facade";
import "@univerjs/sheets-filter/facade";
import "@univerjs/sheets-conditional-formatting/facade";
import "@univerjs/sheets-data-validation/facade";
import "@univerjs/sheets-sort/facade";
import "@univerjs/sheets-drawing-ui/facade";

export interface UniverConfig {
  container: HTMLElement;
  locale?: LocaleType;
}

export interface UniverInstance {
  univer: Univer;
  univerAPI: FUniver;
}

export function createUniverInstance(config: UniverConfig): UniverInstance {
  const locale = config.locale ?? LocaleType.EN_US;

  const univer = new Univer({
    locale,
    locales: {
      [LocaleType.EN_US]: mergeLocales(
        DesignEnUS,
        UIEnUS,
        DocsUIEnUS,
        SheetsEnUS,
        SheetsUIEnUS,
        SheetsFormulaUIEnUS,
        SheetsNumfmtUIEnUS,
        SheetsFilterUIEnUS,
        SheetsConditionalFormattingUIEnUS,
        SheetsDataValidationUIEnUS,
        SheetsSortUIEnUS,
        DrawingUIEnUS,
        SheetsDrawingUIEnUS
      ),
    },
  });

  // Core plugins - load order matters!
  univer.registerPlugin(UniverRenderEnginePlugin);
  univer.registerPlugin(UniverFormulaEnginePlugin);
  univer.registerPlugin(UniverUIPlugin, { container: config.container });

  // Document support for rich text cells
  univer.registerPlugin(UniverDocsPlugin);
  univer.registerPlugin(UniverDocsUIPlugin);

  // Core spreadsheet functionality
  univer.registerPlugin(UniverSheetsPlugin);
  univer.registerPlugin(UniverSheetsUIPlugin);

  // Formula support
  univer.registerPlugin(UniverSheetsFormulaPlugin);
  univer.registerPlugin(UniverSheetsFormulaUIPlugin);

  // Number formatting
  univer.registerPlugin(UniverSheetsNumfmtPlugin);
  univer.registerPlugin(UniverSheetsNumfmtUIPlugin);

  // Filter and sort (Phase 12)
  univer.registerPlugin(UniverSheetsFilterPlugin);
  univer.registerPlugin(UniverSheetsFilterUIPlugin);
  univer.registerPlugin(UniverSheetsSortPlugin);
  univer.registerPlugin(UniverSheetsSortUIPlugin);

  // Conditional formatting (Phase 13)
  univer.registerPlugin(UniverSheetsConditionalFormattingPlugin);
  univer.registerPlugin(UniverSheetsConditionalFormattingUIPlugin);

  // Data validation (Phase 10)
  univer.registerPlugin(UniverDataValidationPlugin);
  univer.registerPlugin(UniverSheetsDataValidationPlugin);
  univer.registerPlugin(UniverSheetsDataValidationUIPlugin);

  // Drawing support for images (Phase 9)
  // IMPORTANT: All @univerjs/* packages must be at the same version.
  // A version mismatch (e.g. drawing at 0.15.3 vs core at 0.15.2) causes
  // silent DI resolution failures at runtime. See appendix_d_univer_integration.md.
  //
  // The `as never` casts are needed because TypeScript cannot see the
  // protected _injector field across package boundaries. This is a
  // compile-time-only cast with no runtime effect.
  univer.registerPlugin(UniverDrawingPlugin as never);
  univer.registerPlugin(UniverDrawingUIPlugin as never);
  univer.registerPlugin(UniverSheetsDrawingPlugin as never);
  univer.registerPlugin(UniverSheetsDrawingUIPlugin as never);

  const univerAPI = FUniver.newAPI(univer);

  return { univer, univerAPI };
}

export function disposeUniverInstance(instance: UniverInstance): void {
  instance.univer.dispose();
}
