import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import {
  LocaleType,
  mergeLocales,
  Univer,
  UniverInstanceType,
} from "@univerjs/core";
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

export interface TomlTableData {
  headers: string[];
  rows: (string | number | boolean | null)[][];
}

export interface UniverSpreadsheetHandle {
  getData: () => TomlTableData | null;
}

interface UniverSpreadsheetProps {
  width?: string | number;
  height?: string | number;
  data?: TomlTableData;
  onChange?: (data: TomlTableData) => void;
}

export const UniverSpreadsheet = forwardRef<
  UniverSpreadsheetHandle,
  UniverSpreadsheetProps
>(function UniverSpreadsheet(
  { width = "100%", height = "600px", data, onChange },
  ref
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const univerRef = useRef<Univer | null>(null);
  const univerAPIRef = useRef<FUniver | null>(null);
  const headersRef = useRef<string[]>([]);
  const onChangeRef = useRef(onChange);
  const isLoadingRef = useRef(false);

  onChangeRef.current = onChange;

  const extractData = (): TomlTableData | null => {
    const sheet = univerAPIRef.current?.getActiveWorkbook()?.getActiveSheet();
    if (!sheet || headersRef.current.length === 0) return null;

    const headers = headersRef.current;
    const rows: (string | number | boolean | null)[][] = [];
    let rowIndex = 2;
    let hasData = true;

    while (hasData) {
      const row: (string | number | boolean | null)[] = [];
      let rowHasContent = false;

      for (let colIndex = 0; colIndex < headers.length; colIndex++) {
        const colLetter = getColumnLetter(colIndex);
        const cellAddress = `${colLetter}${rowIndex}`;
        const cellValue = sheet.getRange(cellAddress)?.getValue();

        if (cellValue !== undefined && cellValue !== null && cellValue !== "") {
          rowHasContent = true;
          row.push(cellValue as string | number | boolean);
        } else {
          row.push(null);
        }
      }

      if (rowHasContent) {
        rows.push(row);
        rowIndex++;
      } else {
        hasData = false;
      }
    }

    return { headers, rows };
  };

  useImperativeHandle(ref, () => ({
    getData: extractData,
  }));

  useEffect(() => {
    if (!containerRef.current || univerRef.current) return;

    const univer = new Univer({
      locale: LocaleType.EN_US,
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
          SheetsSortUIEnUS
        ),
      },
    });

    univerRef.current = univer;

    univer.registerPlugin(UniverRenderEnginePlugin);
    univer.registerPlugin(UniverFormulaEnginePlugin);
    univer.registerPlugin(UniverUIPlugin, { container: containerRef.current });
    univer.registerPlugin(UniverDocsPlugin);
    univer.registerPlugin(UniverDocsUIPlugin);
    univer.registerPlugin(UniverSheetsPlugin);
    univer.registerPlugin(UniverSheetsUIPlugin);
    univer.registerPlugin(UniverSheetsFormulaPlugin);
    univer.registerPlugin(UniverSheetsFormulaUIPlugin);
    univer.registerPlugin(UniverSheetsNumfmtPlugin);
    univer.registerPlugin(UniverSheetsNumfmtUIPlugin);
    univer.registerPlugin(UniverSheetsFilterPlugin);
    univer.registerPlugin(UniverSheetsFilterUIPlugin);
    univer.registerPlugin(UniverSheetsConditionalFormattingPlugin);
    univer.registerPlugin(UniverSheetsConditionalFormattingUIPlugin);
    univer.registerPlugin(UniverDataValidationPlugin);
    univer.registerPlugin(UniverSheetsDataValidationPlugin);
    univer.registerPlugin(UniverSheetsDataValidationUIPlugin);
    univer.registerPlugin(UniverSheetsSortPlugin);
    univer.registerPlugin(UniverSheetsSortUIPlugin);

    univer.createUnit(UniverInstanceType.UNIVER_SHEET, {});

    const univerAPI = FUniver.newAPI(univer);
    univerAPIRef.current = univerAPI;

    univerAPI.onCommandExecuted((command) => {
      if (isLoadingRef.current) return;
      if (
        command.id === "sheet.mutation.set-range-values" ||
        command.id === "sheet.command.set-range-values"
      ) {
        const newData = extractData();
        if (newData && onChangeRef.current) {
          onChangeRef.current(newData);
        }
      }
    });

    return () => {
      univer.dispose();
      univerRef.current = null;
      univerAPIRef.current = null;
    };
  }, []);

  useEffect(() => {
    const sheet = univerAPIRef.current?.getActiveWorkbook()?.getActiveSheet();
    if (!sheet || !data) return;

    isLoadingRef.current = true;
    headersRef.current = data.headers;

    data.headers.forEach((header, colIndex) => {
      const colLetter = getColumnLetter(colIndex);
      const range = sheet.getRange(`${colLetter}1`);
      range?.setValue(header);
      range?.setFontWeight("bold");
    });

    data.rows.forEach((row, rowIndex) => {
      row.forEach((cellValue, colIndex) => {
        const colLetter = getColumnLetter(colIndex);
        const cellAddress = `${colLetter}${rowIndex + 2}`;
        const displayValue = cellValue === null ? "" : cellValue;
        sheet.getRange(cellAddress)?.setValue(displayValue);
      });
    });

    isLoadingRef.current = false;
  }, [data]);

  return (
    <div
      ref={containerRef}
      style={{
        width: typeof width === "number" ? `${width}px` : width,
        height: typeof height === "number" ? `${height}px` : height,
      }}
    />
  );
});

function getColumnLetter(index: number): string {
  let result = "";
  let n = index;
  while (n >= 0) {
    result = String.fromCharCode((n % 26) + 65) + result;
    n = Math.floor(n / 26) - 1;
  }
  return result;
}
