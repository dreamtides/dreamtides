# Univer Data Validation Reference

Comprehensive guide for implementing data validation rules in Univer spreadsheets.

## Setup

Data validation is included in the core sheets preset. For manual setup:

```typescript
import { UniverDataValidationPlugin } from '@univerjs/data-validation';
import { UniverSheetsDataValidationPlugin } from '@univerjs/sheets-data-validation';
```

## Creating Validation Rules

Use `univerAPI.newDataValidation()` to create a builder:

```typescript
const rule = univerAPI.newDataValidation()
  .requireValueInList(['Option1', 'Option2'])
  .setOptions({ allowBlank: true })
  .build();

fRange.setDataValidation(rule);
```

## Validation Types

### Dropdown List (from values)
```typescript
const rule = univerAPI.newDataValidation()
  .requireValueInList(['Yes', 'No', 'Maybe'], multiple, showDropdown)
  .build();
```
- `multiple` (optional): Allow multiple selections
- `showDropdown` (optional): Show dropdown arrow (default: true)

### Dropdown List (from range)
```typescript
const sourceRange = fWorksheet.getRange('Z1:Z10');
const rule = univerAPI.newDataValidation()
  .requireValueInRange(sourceRange, multiple, showDropdown)
  .build();
```

### Checkbox
```typescript
// Default values: checked=1, unchecked=0
const rule = univerAPI.newDataValidation()
  .requireCheckbox()
  .build();

// Custom values
const rule = univerAPI.newDataValidation()
  .requireCheckbox('Yes', 'No')
  .build();
```

### Number Validation
```typescript
// Between range
.requireNumberBetween(1, 100, isInteger?)

// Not between
.requireNumberNotBetween(1, 100, isInteger?)

// Comparisons
.requireNumberGreaterThan(10, isInteger?)
.requireNumberGreaterThanOrEqualTo(10, isInteger?)
.requireNumberLessThan(10, isInteger?)
.requireNumberLessThanOrEqualTo(10, isInteger?)
.requireNumberEqualTo(10, isInteger?)
.requireNumberNotEqualTo(10, isInteger?)
```
- `isInteger` (optional): Require whole numbers only

### Date Validation
```typescript
// Date range
.requireDateBetween(new Date('2024-01-01'), new Date('2024-12-31'))
.requireDateNotBetween(startDate, endDate)

// Comparisons
.requireDateAfter(new Date('2024-01-01'))
.requireDateBefore(new Date('2024-12-31'))
.requireDateOnOrAfter(date)
.requireDateOnOrBefore(date)
.requireDateEqualTo(date)
```

### Custom Formula
```typescript
const rule = univerAPI.newDataValidation()
  .requireFormulaSatisfied('=A1>B1')
  .build();
```
- Formula must evaluate to TRUE or FALSE
- References are relative (adjust per cell in range)

## Builder Methods

### setOptions()
```typescript
.setOptions({
  allowBlank: true,        // Allow empty cells
  showErrorMessage: true,  // Show error on invalid input
  error: 'Invalid value',  // Error message text
  showInputMessage: true,  // Show input prompt
  inputMessage: 'Enter a value between 1-100'
})
```

### setAllowBlank()
```typescript
.setAllowBlank(true)  // Allow empty cells to pass validation
```

### setAllowInvalid()
```typescript
// Allow invalid data with warning
.setAllowInvalid(true)  // Shows warning but allows entry

// Strict mode - reject invalid data
.setAllowInvalid(false)  // Prevents invalid entry
```

## Applying Validation

### To a Range
```typescript
const fRange = fWorksheet.getRange('A1:A100');
fRange.setDataValidation(rule);
```

### Get Existing Validation
```typescript
const validation = fRange.getDataValidation();
if (validation) {
  const criteriaType = validation.getCriteriaType();
  const [operator, formula1, formula2] = validation.getCriteriaValues();
}
```

### Clear Validation
```typescript
fRange.clearDataValidation();
```

## Checking Validation Status

```typescript
// Get validation status for all cells in range
const status = await fRange.getValidatorStatus();
// Returns 2D array: [['valid', 'invalid'], ['valid', 'valid']]

// Status values: 'valid' | 'invalid' | 'blank'
```

## Complete Examples

### Dropdown with Error Message
```typescript
const fWorkbook = univerAPI.getActiveWorkbook();
const fWorksheet = fWorkbook.getActiveSheet();

const rule = univerAPI.newDataValidation()
  .requireValueInList(['Red', 'Green', 'Blue'])
  .setOptions({
    allowBlank: false,
    showErrorMessage: true,
    error: 'Please select a color from the list'
  })
  .build();

fWorksheet.getRange('A1:A100').setDataValidation(rule);
```

### Number Range with Integer Requirement
```typescript
const rule = univerAPI.newDataValidation()
  .requireNumberBetween(1, 100, true) // integers only
  .setAllowBlank(false)
  .setAllowInvalid(false) // strict mode
  .setOptions({
    showErrorMessage: true,
    error: 'Enter a whole number between 1 and 100'
  })
  .build();

fWorksheet.getRange('B1:B100').setDataValidation(rule);
```

### Checkbox Column
```typescript
const rule = univerAPI.newDataValidation()
  .requireCheckbox('Complete', 'Pending')
  .build();

fWorksheet.getRange('C1:C100').setDataValidation(rule);
```

### Date Range Validation
```typescript
const rule = univerAPI.newDataValidation()
  .requireDateBetween(
    new Date('2024-01-01'),
    new Date('2024-12-31')
  )
  .setOptions({
    showErrorMessage: true,
    error: 'Date must be within 2024'
  })
  .build();

fWorksheet.getRange('D1:D100').setDataValidation(rule);
```

### Formula-Based Validation
```typescript
// Cell value must be greater than the cell to its left
const rule = univerAPI.newDataValidation()
  .requireFormulaSatisfied('=B1>A1')
  .setOptions({
    showErrorMessage: true,
    error: 'Value must be greater than column A'
  })
  .build();

fWorksheet.getRange('B1:B100').setDataValidation(rule);
```

### Copy and Modify Rules
```typescript
const builder = univerAPI.newDataValidation()
  .requireNumberBetween(1, 10)
  .setOptions({ showErrorMessage: true });

// Apply to first range
fWorksheet.getRange('A1:A10').setDataValidation(builder.build());

// Copy and modify for second range
const newBuilder = builder.copy()
  .requireNumberBetween(1, 100); // Different range

fWorksheet.getRange('B1:B10').setDataValidation(newBuilder.build());
```

## DataValidationType Enum

```typescript
// Access via univerAPI.Enum.DataValidationType
enum DataValidationType {
  CUSTOM = 'custom',           // Custom formula
  LIST = 'list',               // Dropdown (single select)
  LIST_MULTIPLE = 'listMultiple', // Dropdown (multi-select)
  CHECKBOX = 'checkbox',       // Checkbox
  DECIMAL = 'decimal',         // Decimal number
  WHOLE = 'whole',             // Integer
  DATE = 'date',               // Date
  TEXT_LENGTH = 'textLength',  // Text length
}
```

## DataValidationOperator Enum

```typescript
enum DataValidationOperator {
  BETWEEN = 'between',
  NOT_BETWEEN = 'notBetween',
  EQUAL = 'equal',
  NOT_EQUAL = 'notEqual',
  GREATER_THAN = 'greaterThan',
  GREATER_THAN_OR_EQUAL = 'greaterThanOrEqual',
  LESS_THAN = 'lessThan',
  LESS_THAN_OR_EQUAL = 'lessThanOrEqual',
}
```

## Error Styles

```typescript
enum DataValidationErrorStyle {
  STOP = 'stop',      // Reject invalid input
  WARNING = 'warning', // Allow with warning
  INFO = 'info'       // Allow with info message
}
```

Use `setAllowInvalid(false)` for STOP behavior, `setAllowInvalid(true)` for WARNING.
