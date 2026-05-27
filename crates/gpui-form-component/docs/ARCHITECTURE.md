# gpui-form-component Architecture

`gpui-form-component` is the GPUI runtime layer for the workspace.

It owns the runtime contracts and helper types that generated forms reference
once macro expansion is complete.

## Purpose

This crate exists for behavior that cannot live purely in proc-macro output or
schema metadata:

- localized date-picker runtime state
- native file-picker runtime state over GPUI path prompts
- cascading select runtime helpers for nested enums
- the runtime contract for component shape state

## Modules

- `src/lib.rs`: public module surface
- `src/shape.rs`: `ComponentShape` and `component_shape!`
- `src/infinite_select.rs`: `InfiniteSelect`, `InfiniteSelectItem`,
  `InfiniteSelectPath`, `Select`, and path reconstruction helpers
- `src/date_picker.rs`: runtime state and element wrapper for localized date
  editing
- `src/calendar.rs`: private calendar popover used by `date_picker`, with
  ICU4X-driven labels and locale-specific week layout
- `src/file_picker.rs`: runtime state and element wrapper for native path
  selection with `gpui::PathPromptOptions`
- `src/i18n.rs`: crate-local `es-fluent` message enums plus helpers for
  caller-owned localizers

## Subsystem Boundaries

### `shape`

`ComponentShape` is the contract targeted by `component = Shape`
expressions.

Responsibilities:

- define the state type that generated forms store in `FormFields`
- define how that state type is constructed
- optionally carry a UI component path for prototyping output
- optionally carry prototyping preferences such as the generated field/helper
  suffix
- optionally implement `ComponentValueBinding<T>` so generated
  prototyping code can seed state and map component events back into
  `FormValueChange<T>`
  through helper aliases/functions instead of exposing associated-type
  projections at every generated call site

### `infinite_select`

This subsystem provides the runtime half of `gpui_form_component::InfiniteSelect`.

Responsibilities:

- represent nested enum variant choices as selectable runtime items
- require `PartialEq` on `InfiniteSelect` values so the backing
  `gpui-component` select can compare current and candidate selections
- track confirmed selection indices with `InfiniteSelectPath`
- track stable persisted selections with `InfiniteSelectKeyPath`
- serialize stable key paths to and from strings for persistence
- report invalid stored paths with `InfiniteSelectPathError`
- own the cascading root/child `SelectState`s through `Select`
- implement the component shape on `Select` itself when the
  `derive` feature is enabled
- expose render-ready `InfiniteSelectLevel` / `InfiniteSelectSnapshot` views and
  `form_fields()` helpers plus `InfiniteSelectField` for form code
- reconstruct nested enum values from stored paths
- emit `InfiniteSelectEvent<T>` with previous/current value state, both path
  forms, and the changed depth
- expose type/child labels for generated and prototyped UI

### `date_picker`

This subsystem wraps calendar behavior in a form-oriented API and owns a
private calendar popover so the date picker can use one ICU4X locale for both
the selected-date label and the calendar chrome.

Responsibilities:

- hold selected date state in `DatePickerState`
- hold selected manual date-range state in `DateRangePickerState`
- emit `DatePickerEvent::Change(Option<jiff::civil::Date>)`
- emit `DateRangePickerEvent::Change(Option<jiff::civil::Date>, Option<jiff::civil::Date>)`
- format display text with locale-aware ICU4X/Jiff formatting
- format calendar month names, weekday headers, day/year labels, and week-start
  layout with ICU4X locale data
- provide plain English fallback placeholder copy and let applications pass
  localized placeholder text explicitly
- keep generated code independent from `chrono` display formatting details

### `file_picker`

This subsystem wraps GPUI's native platform path prompt in a form-oriented API.

Responsibilities:

- hold selected path state in `FilePickerState`
- emit `FilePickerEvent::Change`, `Cancel`, and `Error`
- render the control with `gpui-component` buttons, icons, theme tokens, and
  sizing helpers
- provide plain English fallback copy for default placeholders, prompts,
  button labels, selected-count text, and dropped-dialog errors while keeping
  Fluent resources available for caller-owned localizers
- use the workspace-pinned GPUI git API instead of adding another native dialog
  dependency

## Data Flow

### Infinite select

1. `gpui-form-component-derive` generates an `InfiniteSelect` impl for a user
   enum. Applications import it through `gpui-form-component` with the `derive`
   feature or directly from `gpui-form-component-derive`.
1. `InfiniteSelect<T>` constructs the master select, derives child selects,
   keeps both `InfiniteSelectPath` and `InfiniteSelectKeyPath` aligned with the
   current nested value, can snapshot the visible levels for rendering, and can
   emit ready-to-render form fields directly.
1. Generated or prototyped form code subscribes to
   `InfiniteSelectEvent<T>` and uses `form_fields()` / `snapshot()` instead of
   managing child-select rebuilds.
1. `build_from_path`, `build_from_key_path`, `path_from_value`, and
   `key_path_from_value` convert between concrete values and stored paths when
   callers need standalone conversion; invalid persisted paths return
   `InfiniteSelectPathError`, while string persistence can use
   `InfiniteSelectKeyPath`'s `Display` / `FromStr`.

### Component shapes

1. Users either declare a wrapper shape with `gpui_form_derive::component_shape!`
   or derive `ComponentShape` directly on owned state. The runtime module also
   keeps a simple `component_shape!` helper for local runtime shapes.
1. `GpuiForm` uses that shape to emit `FormFields` entity state and
   `FormComponents` constructors.
1. Schema/prototyping metadata can optionally carry a concrete UI component path
   for scaffold generation.
1. Shape-level prototyping metadata can carry a preferred field/helper suffix
   for scaffold generation, with field-level annotations able to override it.
1. When the field opts into `value_binding`, prototyping code calls the
   shape-owned `ComponentValueBinding<T>` hooks instead of inferring any
   domain-specific event semantics; generated code can route those calls
   through `ComponentStateOf`, `ComponentEventOf`,
   `seed_value_binding_state`, `form_value_change`, and `FormValueChange<T>`.
   `ComponentEventOf` resolves to the binding's associated `Event`, so
   owned states can expose their own event enum and external wrappers can keep
   their upstream event type.

### Date picker

1. Component shapes can store `Entity<DatePickerState>` in `FormFields`.
1. Runtime date selection emits `DatePickerEvent::Change`.
1. Shape-owned value adapters can convert the `jiff::civil::Date` into the
   holder field type with `parse_form_date` and any `type`/`into` conversion
   hooks.
1. Manual range-picking UI can store `Entity<DateRangePickerState>`, render
   `DateRangePicker`, and subscribe to `DateRangePickerEvent::Change`.

### File picker

1. Manual form code stores `Entity<FilePickerState>`.
1. `FilePicker` renders a path display, clear action, and browse button.
1. Browse actions call `App::prompt_for_paths(PathPromptOptions)` and update the
   state asynchronously when the platform dialog returns.
1. Subscribers receive changed path lists, cancellation, or platform-dialog
   errors through `FilePickerEvent`.

### Built-in text

1. `src/i18n.rs` defines this crate's embedded Fluent resource module and
   message enums for caller-owned `es-fluent` localizers.
1. `i18n.toml` allowlists the runtime namespaces (`date_picker`,
   `file_picker`).
1. Fluent resources live under
   `i18n/{locale}/gpui-form-component/{namespace}.ftl`; add new component text
   to the matching namespace file instead of a shared crate-level Fluent file.
1. Runtime resources currently ship for `en`, `fr-FR`, and `zh-CN`.
1. Runtime components use plain English fallback copy unless callers pass
   localized text explicitly; `src/i18n.rs` exposes helpers that render this
   crate's messages through caller-owned localizers.
1. Caller-provided labels, prompts, placeholders, and event errors remain
   caller-owned text.
1. Story/demo text belongs to `gpui-form-component-story`, not this runtime
   crate.

## Dependency Role

This crate should remain focused on runtime GPUI behavior.

It should not own:

- derive-time parsing rules
- inventory registration
- schema metadata definitions

Those belong in `gpui-form-codegen`, `gpui-form-derive`, and
`gpui-form-schema`.

## Coordination Rules

When adding a new reusable component shape that needs runtime state:

1. add the runtime helper in this crate
1. expose it through `ComponentShape` or a helper macro/derive
1. publish shape metadata such as `COMPONENT_PATH`, `VALUE_BINDING`, and
   `PROTOTYPING.field_suffix` when generated scaffolds need it
1. update user-facing docs so applications import the runtime crate explicitly

## When To Update This Document

Update this file when:

- runtime responsibilities move between modules
- a new runtime helper module is added
- the component shape contract changes
- infinite-select or date-picker event/data flow changes
- story/demo ownership moves back into or out of this runtime crate
