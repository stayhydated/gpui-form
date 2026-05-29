# gpui-form-derive Architecture

`gpui-form-derive` owns the main proc-macro entry points for the workspace.

It parses user structs and enums, delegates component modeling to
`gpui-form-codegen`, and emits the generated types and inventory metadata that
power the rest of the ecosystem.

## Entry Points

`src/lib.rs` defines:

- `#[derive(GpuiForm)]`
- `#[derive(ComponentShape)]`
- `component_shape!`
- `#[gpui_form_derive::component_value_binding]`

`#[derive(InfiniteSelect)]` is not part of this crate. It lives in
`gpui-form-component-derive` and is re-exported by `gpui-form-component` when
that crate's `derive` feature is enabled.

`#[derive(SelectItem)]` is also not part of this crate. It lives in
`gpui-form-collection-derive`.

## Module Layout

- `src/derives/gpui_form/mod.rs`: `GpuiForm` entry module
- `src/derives/gpui_form/expansion.rs`: top-level `GpuiForm` expansion pipeline
- `src/derives/gpui_form/components.rs`: delegates component fields into
  codegen layouts
- `src/derives/gpui_form/value_holder.rs`: generated holder types, defaults,
  conversion logic, and skip-field handling
- `src/derives/gpui_form/koruma.rs`: Koruma metadata mirroring helpers
- `src/derives/gpui_form/cfg_attr.rs`: `cfg_attr` flattening before parse-time
  inspection
- `src/derives/component_shape_metadata.rs`: shared `ComponentShape` metadata
  parsing state and emitted associated-const helpers for shape declaration
  macros
- `src/derives/component_shape_state.rs`: `ComponentShape` derive expansion
- `src/derives/component_shape.rs`: function-like component shape
  expansion for local wrappers around external component/state types
- `src/derives/component_value_binding.rs`: attribute macro expansion that
  rewrites backing-state value bindings for component-derived shapes

## `GpuiForm` Expansion Pipeline

1. Parse the input with `syn`.
1. Flatten `cfg_attr` wrappers so downstream parsing sees effective
   `#[gpui_form(...)]` data.
1. Parse struct-level and field-level `#[gpui_form(...)]` data with `darling`.
   Field-level component shapes are parsed into a typed shape path from
   `#[gpui_form(component(shape = my::Shape))]`;
   arbitrary Rust expressions are only parsed for `default`, `source_to_form`,
   and `form_to_source`. The parser rejects duplicate component expressions
   before codegen and rejects field-level component metadata such as
   `component = ...` or `field_suffix = ...`; those belong on the shape
   declaration.
1. Parse Koruma field metadata through `koruma-derive-core`.
1. For each component field, delegate component-specific modeling to
   `gpui-form-codegen`.
1. Emit:
   - `FormFields`
   - `FormComponents`
   - `FormValueHolder`
   - conversions between the original type and the holder
   - optional inventory metadata

## Value Holder Responsibilities

`value_holder.rs` owns:

- optionality normalization between model fields and editable form state
- default-value seeding
- `type`/`source_to_form`/`form_to_source` conversions
- `#[gpui_form(skip)]` reconstruction behavior
- Koruma mirroring for holder validation

Important behaviors:

- originally optional fields stay optional in the holder
- input-style fields usually wrap in `Option<T>` to represent empty UI state
- skipped fields are still prefilled when converting from the original model
  into the holder
- reverse conversion becomes explicit `into_original(...)` when skipped fields
  prevent a fully automatic round trip
- every field is normalized as component, hidden, or skipped intent during
  parsing; missing intent is a compile error so a field cannot silently become
  value-holder-only
- hidden fields are explicit value-holder-only fields and may use default and
  conversion metadata
- skipped fields reject component, hidden, default, and conversion metadata
  during field parsing
- expansion builds one `AnalyzedField` list after component field generation;
  inventory metadata, value-holder generation, type checks, validation wiring,
  defaults, conversions, and shape requiredness all read from that list instead
  of recomputing field type facts independently
- holder-to-model conversion uses `TryFrom` when any non-skipped field can be
  missing without a default; `From` is reserved for holders the derive can prove
  infallible directly
- shape-policy holder conversion keeps the checked `try_into_original()` path
  when requiredness is inherited from the shape policy. A field-level default
  makes that field statically infallible and lets the normal infallible holder
  path apply.
- non-optional shape-backed fields whose `ValueStoragePolicy` can represent
  missing values also get generated Koruma validators so `validate()` reports
  missing shape-required values before holder-to-model conversion

## Koruma Integration

`GpuiForm` can enable Koruma-aware holder generation even when the source
struct only contains field-level `#[koruma(...)]` attributes.

The derive layer:

- reads normalized validator metadata from `koruma-derive-core`
- mirrors validators into the holder type
- preserves validator path and builder-chain forms
- injects required-value behavior where holder optionality would otherwise lose
  source-model required semantics

## Inventory Integration

When the `inventory` feature is enabled:

1. `GpuiForm` emits one `GpuiFormShape` per non-generic derived struct, and
   generic source structs must opt out with `#[gpui_form(no_inventory)]` when
   the `inventory` feature is enabled. Inventory items must name one concrete
   source type and module path; the generated form code still carries the source
   generics.
1. Each field becomes a `FieldVariant` with component contract metadata from
   `gpui-form-codegen`.
1. Metadata includes validation rule identifiers, defaults, full value type
   paths, component shape UI paths, and skipped-field information for
   downstream generators.

## Other Derives

### `ComponentShape`

- emits a `ComponentShape` impl for a rendered component when `state = ...`
  supplies separate backing state
- defaults constructor wiring to `<State>::new(window, cx)`
- accepts a constructor expression for `new = ...`; paths and closures receive
  `(window, cx)`, while direct constructor expressions are emitted as written
- optionally stores a component type for prototyping output
- sets shape-level `ValueBindingPolicy` only when `value_binding` is present,
  for `ComponentValueBinding<T>` prototyping hooks, including
  `seed_value_binding_state` and `form_value_change`
- emits component-derived `ComponentValueBinding<T>` delegation through the
  backing state's `ComponentStateValueBinding<T>` implementation only when
  `value_binding` is present
- defaults to implementing `gpui_form_runtime::shape::ComponentShape`
  for downstream application or integration crates that define shapes directly
- resolves generated `gpui`, `gpui-form`, and `gpui-form-runtime` paths through
  the shared codegen crate-path resolver

### `component_shape!`

- emits a local zero-sized shape type plus `ComponentShape` impl
- accepts caller generics, where clauses, and outer attributes
- accepts `new`, `component`, `value_storage = require_value|direct`,
  `value_binding`, and `field_suffix` metadata keys
- uses semicolon separators between metadata entries
- accepts nested `impl` items and emits them after the generated
  `ComponentShape` impl
- sets shape-level `ValueBindingPolicy` only when `value_binding;` is present
- defaults omitted `new` metadata to `<State>::new(window, cx)`
- emits a broad `ComponentShapeFor<Value>` impl unless the block contains an
  explicit `ComponentShapeFor` impl
- targets external component/state pairs that cannot directly implement
  `ComponentShape` because both the trait and state type are foreign
- emits the implementation against `gpui_form_runtime::shape` by default
- leaves optional external `ComponentValueBinding<T>` implementations to the
  caller or collection crate when they are not nested in the macro block

These shape-definition macros are lower-level than `#[derive(GpuiForm)]` and
still require a direct `gpui-form-runtime` dependency. Generated `GpuiForm`
component fields use the facade path `gpui_form::runtime::shape`.

## Coordination Rules

When adding a component or attribute:

1. update parse-time option support in `gpui-form-codegen`
1. update holder behavior in this crate if optionality or conversion changes
1. update `gpui-form-schema` metadata emission
1. update `gpui-form-prototyping-core` generator support
1. update user-facing docs in the facade README and derive README

If the change also affects `InfiniteSelect`, update
`gpui-form-component-derive`, `gpui-form-component`, and their docs in the same
change.

## Tests

- targeted `GpuiForm` expansion tests live in
  `src/derives/gpui_form/tests.rs`
- compile-fail UI tests live under `tests/ui`

## When To Update This Document

Update this file when:

- the expansion pipeline changes
- holder conversion behavior changes
- inventory or Koruma emission rules change
- macro responsibilities move between modules
