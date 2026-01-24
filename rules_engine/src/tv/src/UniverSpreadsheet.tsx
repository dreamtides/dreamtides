import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { Univer, UniverInstanceType } from "@univerjs/core";
import { FUniver } from "@univerjs/core/facade";

import {
  createUniverInstance,
  disposeUniverInstance,
  UniverInstance,
} from "./univer_config";
import type { TomlTableData } from "./ipc_bridge";

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

    const instance: UniverInstance = createUniverInstance({
      container: containerRef.current,
    });
    univerRef.current = instance.univer;
    univerAPIRef.current = instance.univerAPI;

    instance.univer.createUnit(UniverInstanceType.UNIVER_SHEET, {});

    instance.univerAPI.onCommandExecuted((command) => {
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
      disposeUniverInstance(instance);
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
