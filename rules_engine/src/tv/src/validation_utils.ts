import { FUniver } from "@univerjs/core/facade";
import { FWorksheet } from "@univerjs/sheets/facade";

import type { TomlTableData, EnumValidationInfo } from "./ipc_bridge";
import { getColumnLetter } from "./header_utils";
import { createLogger } from "./logger_frontend";

const logger = createLogger("tv.ui.validation");

/**
 * Detect columns that contain only boolean values (or nulls).
 * Returns an array of column indices that should be rendered as checkboxes.
 */
export function detectBooleanColumns(data: TomlTableData): number[] {
  const booleanColumns: number[] = [];

  for (let colIdx = 0; colIdx < data.headers.length; colIdx++) {
    let hasNonNullValue = false;
    let allBoolean = true;

    for (const row of data.rows) {
      const value = row[colIdx];
      if (value === null || value === undefined) {
        continue;
      }
      hasNonNullValue = true;
      if (typeof value !== "boolean") {
        allBoolean = false;
        break;
      }
    }

    if (hasNonNullValue && allBoolean) {
      booleanColumns.push(colIdx);
    }
  }

  return booleanColumns;
}

/**
 * Apply checkbox data validation to boolean columns.
 * Uses Univer's data validation API to render checkboxes for boolean values.
 */
export function applyCheckboxValidation(
  univerAPI: FUniver,
  sheet: FWorksheet,
  data: TomlTableData,
  booleanColumns: number[],
  dataOffset: number = 0,
): void {
  if (booleanColumns.length === 0 || data.rows.length === 0) {
    return;
  }

  for (const colIdx of booleanColumns) {
    const colLetter = getColumnLetter(colIdx + dataOffset);
    const startRow = 2;
    const endRow = data.rows.length + 1;
    const rangeAddress = `${colLetter}${startRow}:${colLetter}${endRow}`;
    const range = sheet.getRange(rangeAddress);

    if (range) {
      const rule = univerAPI
        .newDataValidation()
        .requireCheckbox()
        .setOptions({ showErrorMessage: false })
        .build();
      range.setDataValidation(rule);

      logger.debug("Applied checkbox validation to column", {
        column: data.headers[colIdx],
        range: rangeAddress,
      });
    }
  }
}

/**
 * Apply dropdown (list) data validation to enum columns.
 * Uses Univer's data validation API to render dropdowns for enum values with type-ahead filtering.
 */
export function applyDropdownValidation(
  univerAPI: FUniver,
  sheet: FWorksheet,
  data: TomlTableData,
  enumRules: EnumValidationInfo[],
  dataOffset: number = 0,
): void {
  if (enumRules.length === 0 || data.rows.length === 0) {
    return;
  }

  for (const rule of enumRules) {
    const colIdx = data.headers.indexOf(rule.column);
    if (colIdx === -1) {
      logger.debug(
        "Enum column not found in headers, skipping dropdown validation",
        {
          column: rule.column,
        },
      );
      continue;
    }

    const colLetter = getColumnLetter(colIdx + dataOffset);
    const startRow = 2;
    const endRow = data.rows.length + 1;
    const rangeAddress = `${colLetter}${startRow}:${colLetter}${endRow}`;
    const range = sheet.getRange(rangeAddress);

    if (range) {
      const validationRule = univerAPI
        .newDataValidation()
        .requireValueInList(rule.allowed_values, false)
        .setOptions({
          showErrorMessage: true,
          error: `Value must be one of: ${rule.allowed_values.join(", ")}`,
        })
        .build();
      range.setDataValidation(validationRule);

      logger.debug("Applied dropdown validation to column", {
        column: rule.column,
        range: rangeAddress,
        allowedValues: rule.allowed_values,
      });
    }
  }
}
