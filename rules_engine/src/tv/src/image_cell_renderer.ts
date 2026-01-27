import { convertFileSrc } from "@tauri-apps/api/core";
import { ICommandService, ImageSourceType } from "@univerjs/core";
import { FUniver } from "@univerjs/core/facade";
import { IRenderManagerService } from "@univerjs/engine-render";
import {
  convertPositionCellToSheetOverGrid,
  ISheetSelectionRenderService,
  SheetSkeletonManagerService,
} from "@univerjs/sheets-ui";

import type { DerivedResultValue } from "./ipc_bridge";
import * as ipc from "./ipc_bridge";
import { createLogger } from "./logger_frontend";

const logger = createLogger("tv.ui.images");

/** Default image dimensions in pixels for floating images. */
const DEFAULT_IMAGE_WIDTH = 120;
const DEFAULT_IMAGE_HEIGHT = 120;

/** Pixel offset from cell origin for floating images. */
const DEFAULT_COLUMN_OFFSET = 4;
const DEFAULT_ROW_OFFSET = 4;

/** Command IDs for sheet drawing operations. */
const INSERT_SHEET_DRAWING_CMD = "sheet.command.insert-sheet-image";
const REMOVE_SHEET_DRAWING_CMD = "sheet.command.remove-sheet-image";

/** DrawingTypeEnum.DRAWING_IMAGE */
const DRAWING_TYPE_IMAGE = 0;

/** Tracks image state per cell to avoid duplicate insertions. */
type ImageCellState = "loading" | "loaded" | "error";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SheetRef = any;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Injector = any;

function generateDrawingId(): string {
  return Math.random().toString(36).substring(2, 8);
}

/**
 * Manages image rendering in spreadsheet cells using Univer's drawing
 * command system and Tauri's asset protocol for local files.
 *
 * Bypasses the FWorksheet facade methods (newOverGridImage, insertImages)
 * which are broken by Vite's pre-bundling creating duplicate FWorksheet
 * class prototypes. Instead uses univerAPI.executeCommand() with the
 * insert-sheet-image command directly.
 *
 * Drawing commands are registered during Univer's onRendered() lifecycle
 * phase. When image results arrive before onRendered() fires, this class
 * polls until the commands become available rather than registering them
 * manually (which would race with the plugin's lifecycle registration).
 */
export class ImageCellRenderer {
  private univerAPI: FUniver;
  private imageStates: Map<string, ImageCellState> = new Map();
  private imageIds: Map<string, string> = new Map();
  private commandsReady = false;

  constructor(univerAPI: FUniver) {
    this.univerAPI = univerAPI;
  }

  /**
   * Handles a derived result that contains an image path.
   * Converts the local cache path to an asset URL and inserts a floating
   * image at the specified cell position.
   */
  async handleImageResult(
    sheet: SheetRef,
    sheetId: string,
    row: number,
    column: number,
    result: DerivedResultValue
  ): Promise<void> {
    const cellKey = `${sheetId}:${row}:${column}`;

    if (result.type === "image") {
      await this.insertImageAtCell(sheet, cellKey, row, column, result.value);
    } else if (result.type === "error") {
      this.setErrorState(sheet, cellKey, row, column, result.value);
    }
  }

  /**
   * Fetches an image by URL via the backend and inserts it at the cell.
   * Sets a loading placeholder while the fetch is in progress.
   */
  async fetchAndInsertImage(
    sheet: SheetRef,
    sheetId: string,
    row: number,
    column: number,
    imageUrl: string
  ): Promise<void> {
    const cellKey = `${sheetId}:${row}:${column}`;

    if (this.imageStates.get(cellKey) === "loaded") {
      return;
    }

    this.setLoadingState(sheet, cellKey, row, column);

    try {
      const cachePath = await ipc.fetchImage(imageUrl);
      await this.insertImageAtCell(sheet, cellKey, row, column, cachePath);
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      logger.error("Image fetch failed", { cellKey, imageUrl, error: errorMessage });
      this.setErrorState(sheet, cellKey, row, column, errorMessage);
    }
  }

  /**
   * Waits for the drawing plugin's onRendered() lifecycle to register
   * the insert-sheet-image command. Derived image results can arrive
   * before onRendered() fires, so this polls until the command is
   * available rather than registering it manually (which would race
   * with the plugin and cause a duplicate registration error).
   */
  private async waitForCommandsReady(): Promise<boolean> {
    if (this.commandsReady) return true;

    try {
      const injector = (this.univerAPI as Injector)._injector;
      const commandService = injector.get(ICommandService);

      if (commandService.hasCommand(INSERT_SHEET_DRAWING_CMD)) {
        this.commandsReady = true;
        return true;
      }

      for (let i = 0; i < 50; i++) {
        await new Promise((resolve) => setTimeout(resolve, 100));
        if (commandService.hasCommand(INSERT_SHEET_DRAWING_CMD)) {
          this.commandsReady = true;
          logger.info("Drawing commands became available", { waitMs: (i + 1) * 100 });
          return true;
        }
      }

      logger.error("Timed out waiting for drawing commands");
      return false;
    } catch (e) {
      logger.error("Failed to check drawing commands", { error: String(e) });
      return false;
    }
  }

  /**
   * Inserts a floating image at the specified cell using the local cache path.
   * Uses Univer's command system directly instead of facade methods.
   */
  private async insertImageAtCell(
    sheet: SheetRef,
    cellKey: string,
    row: number,
    column: number,
    cachePath: string
  ): Promise<void> {
    try {
      const ready = await this.waitForCommandsReady();
      if (!ready) {
        this.setErrorState(sheet, cellKey, row, column, "Drawing commands not available");
        return;
      }

      await this.removeExistingImage(cellKey, sheet.getSheetId());

      const workbook = this.univerAPI.getActiveWorkbook();
      if (!workbook) {
        this.setErrorState(sheet, cellKey, row, column, "No active workbook");
        return;
      }

      const unitId = workbook.getId();
      const subUnitId = sheet.getSheetId();
      const assetUrl = convertFileSrc(cachePath);

      const injector = (this.univerAPI as Injector)._injector;
      const renderManager = injector.get(IRenderManagerService);
      const renderUnit = renderManager.getRenderById(unitId);
      if (!renderUnit) {
        this.setErrorState(sheet, cellKey, row, column, "Render unit not found");
        return;
      }

      const selectionRenderService = renderUnit.with(ISheetSelectionRenderService);
      const skeletonManager = renderUnit.with(SheetSkeletonManagerService);

      const position = convertPositionCellToSheetOverGrid(
        unitId,
        subUnitId,
        { column, columnOffset: DEFAULT_COLUMN_OFFSET, row, rowOffset: DEFAULT_ROW_OFFSET },
        DEFAULT_IMAGE_WIDTH,
        DEFAULT_IMAGE_HEIGHT,
        selectionRenderService,
        skeletonManager
      );

      const drawingId = generateDrawingId();
      const imageData = {
        drawingId,
        drawingType: DRAWING_TYPE_IMAGE,
        imageSourceType: ImageSourceType.URL,
        source: assetUrl,
        unitId,
        subUnitId,
        sheetTransform: position.sheetTransform,
        transform: position.transform,
      };

      await this.univerAPI.executeCommand(INSERT_SHEET_DRAWING_CMD, {
        unitId,
        drawings: [imageData],
      });

      this.imageIds.set(cellKey, drawingId);
      this.imageStates.set(cellKey, "loaded");

      logger.info("Image inserted at cell", { cellKey, row, column });
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      logger.error("Failed to insert image at cell", {
        cellKey,
        row,
        column,
        error: errorMessage,
      });
      this.setErrorState(sheet, cellKey, row, column, errorMessage);
    }
  }

  /**
   * Removes a previously inserted image for a cell, if any.
   * Uses the remove-sheet-image command directly.
   */
  private async removeExistingImage(
    cellKey: string,
    subUnitId: string
  ): Promise<void> {
    const existingId = this.imageIds.get(cellKey);
    if (!existingId) return;

    try {
      const workbook = this.univerAPI.getActiveWorkbook();
      if (workbook) {
        const unitId = workbook.getId();
        await this.univerAPI.executeCommand(REMOVE_SHEET_DRAWING_CMD, {
          unitId,
          drawings: [{
            unitId,
            drawingId: existingId,
            subUnitId,
            drawingType: DRAWING_TYPE_IMAGE,
          }],
        });
      }
    } catch (e) {
      logger.debug("Could not remove existing image", {
        cellKey,
        drawingId: existingId,
        error: String(e),
      });
    }

    this.imageIds.delete(cellKey);
  }

  /**
   * Sets a loading placeholder in the cell while the image is being fetched.
   */
  private setLoadingState(
    sheet: SheetRef,
    cellKey: string,
    row: number,
    column: number
  ): void {
    this.imageStates.set(cellKey, "loading");
    const range = sheet.getRange(row, column, 1, 1);
    if (range) {
      range.setValue("Loading...");
      range.setFontColor("#999999");
    }
  }

  /**
   * Sets an error placeholder in the cell with a red error indicator.
   */
  private setErrorState(
    sheet: SheetRef,
    cellKey: string,
    row: number,
    column: number,
    errorMessage: string
  ): void {
    this.imageStates.set(cellKey, "error");

    try {
      const range = sheet.getRange(row, column, 1, 1);
      if (range) {
        const truncatedMessage =
          errorMessage.length > 60
            ? errorMessage.substring(0, 57) + "..."
            : errorMessage;
        range.setValue(`[!] ${truncatedMessage}`);
        range.setFontColor("#CC0000");
      }
    } catch (e) {
      logger.error("Failed to set error state in cell", {
        cellKey,
        error: String(e),
      });
    }
  }

  /**
   * Returns the current state of an image cell.
   */
  getImageState(sheetId: string, row: number, column: number): ImageCellState | undefined {
    return this.imageStates.get(`${sheetId}:${row}:${column}`);
  }

  /**
   * Clears all tracked image state for a sheet.
   * Call when sheet data is reloaded to allow fresh image insertion.
   */
  clearSheetImages(_sheet: SheetRef, sheetId: string): void {
    const keysToRemove: string[] = [];
    for (const [key] of this.imageStates) {
      if (key.startsWith(`${sheetId}:`)) {
        keysToRemove.push(key);
      }
    }

    const drawingsToRemove: { unitId: string; drawingId: string; subUnitId: string; drawingType: number }[] = [];
    const workbook = this.univerAPI.getActiveWorkbook();
    const unitId = workbook?.getId();

    for (const key of keysToRemove) {
      const drawingId = this.imageIds.get(key);
      if (drawingId && unitId) {
        drawingsToRemove.push({
          unitId,
          drawingId,
          subUnitId: sheetId,
          drawingType: DRAWING_TYPE_IMAGE,
        });
      }
      this.imageStates.delete(key);
      this.imageIds.delete(key);
    }

    if (drawingsToRemove.length > 0 && unitId) {
      this.univerAPI.executeCommand(REMOVE_SHEET_DRAWING_CMD, {
        unitId,
        drawings: drawingsToRemove,
      }).catch(() => {
        // Ignore cleanup errors
      });
    }

    logger.debug("Cleared sheet images", {
      sheetId,
      clearedCount: keysToRemove.length,
    });
  }

  /**
   * Clears all tracked image state across all sheets.
   */
  clearAll(): void {
    this.imageStates.clear();
    this.imageIds.clear();
  }
}
