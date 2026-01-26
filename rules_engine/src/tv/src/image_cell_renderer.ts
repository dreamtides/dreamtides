import { convertFileSrc } from "@tauri-apps/api/core";
import { ImageSourceType } from "@univerjs/core";

import type { DerivedResultValue } from "./ipc_bridge";
import * as ipc from "./ipc_bridge";

const LOG_TAG = "tv.ui.images";

function logDebug(message: string, data?: unknown): void {
  const entry = {
    level: "DEBUG",
    component: LOG_TAG,
    message,
    data,
    timestamp: new Date().toISOString(),
  };
  console.debug(JSON.stringify(entry));
}

function logInfo(message: string, data?: unknown): void {
  const entry = {
    level: "INFO",
    component: LOG_TAG,
    message,
    data,
    timestamp: new Date().toISOString(),
  };
  console.info(JSON.stringify(entry));
}

function logError(message: string, data?: unknown): void {
  const entry = {
    level: "ERROR",
    component: LOG_TAG,
    message,
    data,
    timestamp: new Date().toISOString(),
  };
  console.error(JSON.stringify(entry));
}

/** Default image dimensions in pixels for floating images. */
const DEFAULT_IMAGE_WIDTH = 120;
const DEFAULT_IMAGE_HEIGHT = 120;

/** Pixel offset from cell origin for floating images. */
const DEFAULT_COLUMN_OFFSET = 4;
const DEFAULT_ROW_OFFSET = 4;

/** Tracks image state per cell to avoid duplicate insertions. */
type ImageCellState = "loading" | "loaded" | "error";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SheetRef = any;

/**
 * Manages image rendering in spreadsheet cells using Univer's floating
 * image (over-grid) support and Tauri's asset protocol for local files.
 */
export class ImageCellRenderer {
  private imageStates: Map<string, ImageCellState> = new Map();
  private imageIds: Map<string, string> = new Map();

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
      logError("Image fetch failed", { cellKey, imageUrl, error: errorMessage });
      this.setErrorState(sheet, cellKey, row, column, errorMessage);
    }
  }

  /**
   * Inserts a floating image at the specified cell using the local cache path.
   */
  private async insertImageAtCell(
    sheet: SheetRef,
    cellKey: string,
    row: number,
    column: number,
    cachePath: string
  ): Promise<void> {
    try {
      await this.removeExistingImage(sheet, cellKey);

      const assetUrl = convertFileSrc(cachePath);

      logDebug("Inserting floating image", {
        cellKey,
        cachePath,
        assetUrl,
        row,
        column,
      });

      const imageBuilder = sheet
        .newOverGridImage()
        .setSource(assetUrl, ImageSourceType.URL)
        .setColumn(column)
        .setRow(row)
        .setColumnOffset(DEFAULT_COLUMN_OFFSET)
        .setRowOffset(DEFAULT_ROW_OFFSET)
        .setWidth(DEFAULT_IMAGE_WIDTH)
        .setHeight(DEFAULT_IMAGE_HEIGHT);

      const imageData = await imageBuilder.buildAsync();
      sheet.insertImages([imageData]);

      const drawingId =
        typeof imageData === "object" && imageData !== null && "drawingId" in imageData
          ? String((imageData as Record<string, unknown>).drawingId)
          : undefined;

      if (drawingId) {
        this.imageIds.set(cellKey, drawingId);
      }

      this.imageStates.set(cellKey, "loaded");

      logInfo("Image inserted at cell", { cellKey, row, column });
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      logError("Failed to insert image at cell", {
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
   */
  private async removeExistingImage(
    sheet: SheetRef,
    cellKey: string
  ): Promise<void> {
    const existingId = this.imageIds.get(cellKey);
    if (!existingId) return;

    try {
      const existingImage = sheet.getImageById(existingId);
      if (existingImage) {
        sheet.deleteImages([existingImage]);
      }
    } catch (e) {
      logDebug("Could not remove existing image", {
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
   * The error message is displayed as cell text for tooltip-on-hover.
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
      logError("Failed to set error state in cell", {
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
  clearSheetImages(sheet: SheetRef, sheetId: string): void {
    const keysToRemove: string[] = [];
    for (const [key] of this.imageStates) {
      if (key.startsWith(`${sheetId}:`)) {
        keysToRemove.push(key);
      }
    }

    for (const key of keysToRemove) {
      const drawingId = this.imageIds.get(key);
      if (drawingId) {
        try {
          const image = sheet.getImageById(drawingId);
          if (image) {
            sheet.deleteImages([image]);
          }
        } catch {
          // Ignore cleanup errors
        }
      }
      this.imageStates.delete(key);
      this.imageIds.delete(key);
    }

    logDebug("Cleared sheet images", {
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
