# Appendix B: Derived Function Architecture

## Function Registry

The function registry is a singleton HashMap initialized at application startup.
It maps string function names to boxed trait objects implementing the
DerivedFunction trait. Registration occurs in the lib.rs initialization before
the Tauri app runs.

## DerivedFunction Trait

The core trait that all derived functions implement:

- name(): Returns the string identifier used in metadata references
- input_keys(): Returns a Vec of TOML key names this function reads
- compute(inputs: &RowData) -> DerivedResult: Performs the computation
- is_async(): Returns whether computation should be offloaded to thread pool

RowData is a HashMap<String, TomlValue> containing all fields from the row.
DerivedResult is an enum with variants for different output types: Text,
Number, Boolean, Image, RichText, Error.

## Computation Lifecycle

When a row changes, the sync layer identifies all derived columns for that row.
For each derived column, it extracts the input values specified by the
function's input_keys(). It then invokes compute() either synchronously or via
the async executor based on is_async(). Results flow back through an event
channel to the frontend.

## Generation Tracking

Each row maintains a generation counter incremented on every change. The
counter value is captured when computation starts. When results arrive, the
counter is checked against current value. Stale results from outdated
generations are discarded to ensure eventual consistency.

## Built-in Function: ImageUrl

The image_url function takes an image_number field and constructs a web URL
by applying a template. The template is stored in metadata configuration.
Output is an Image variant containing the URL string. The image system then
handles fetching and caching.

Input keys: ["image_number"]
Output: DerivedResult::Image(url_string)

## Built-in Function: RulesPreview

The rules_preview function takes rules_text and variables fields, processes
them through Fluent, and produces formatted rich text. It integrates with the
existing localized_strings module from tabula_data. Output is a RichText
variant containing styled spans.

Input keys: ["rules_text", "variables"]
Processing: Parse variables as key-value pairs, build FluentArgs, format
through LocalizedStrings::format_display_string, parse HTML tags for styling
Output: DerivedResult::RichText with bold/italic/color spans for Univer

## Built-in Function: CardLookup

The card_lookup function takes a UUID reference field and searches other
loaded tables for a matching id field, returning the name from that row.
Requires cross-table access which is provided through a lookup context
injected into the compute call.

Input keys: ["referenced_card_id"]
Lookup context: Provides read access to other loaded tables
Output: DerivedResult::Text(card_name) or DerivedResult::Error if not found

## Error Handling

Function panics are caught at the executor boundary. Caught panics produce
DerivedResult::Error with the panic message. Errors display in cells as red
text with the error message. Functions should prefer returning Error variants
over panicking for expected failure cases.

## Async Execution

Async functions run on a dedicated tokio thread pool. The pool size is
configurable, defaulting to the number of CPU cores. Tasks are prioritized by
visibility, with cells currently on screen computed first. Offscreen cells are
computed at lower priority to prepare for scrolling.

## Caching

Computed values are cached by a key combining row index, generation, and
function name. Cache entries are invalidated when the row generation changes.
The cache uses LRU eviction when memory pressure is high.

## Custom Function Registration

Third-party functions can be registered by implementing DerivedFunction and
calling FunctionRegistry::register() during initialization. The function name
must be unique. Duplicate registration panics at startup to surface
configuration errors early.

## Testing Functions

Test utilities provide mock RowData construction and result assertion helpers.
Functions are tested in isolation with synthetic inputs. Integration tests
verify end-to-end flow from cell edit through computation to UI update.
