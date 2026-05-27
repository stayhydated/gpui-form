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

## `GpuiForm` Expansion Pipeline

1. Parse the input with `syn`.
1. Flatten `cfg_attr` wrappers so downstream parsing sees effective
   `#[gpui_form(...)]` data.
1. Parse struct-level and field-level `#[gpui_form(...)]` data with `darling`.
   Field-level component shapes may be written as a positional expression
   (`#[gpui_form(my::Shape)]`) or as `component = my::Shape`; the parser rejects
   duplicate component expressions before codegen.
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
- `type`/`from`/`into` conversions
- `#[gpui_form(skip)]` reconstruction behavior
- Koruma mirroring for holder validation

Important behaviors:

- originally optional fields stay optional in the holder
- input-style fields usually wrap in `Option<T>` to represent empty UI state
- skipped fields are still prefilled when converting from the original model
  into the holder
- reverse conversion becomes explicit `into_original(...)` when skipped fields
  prevent a fully automatic round trip

## Koruma Integration

`GpuiForm` can enable Koruma-aware holder generation even when the source
struct only contains field-level `#[koruma(...)]` attributes.

The derive layer:

- reads normalized validator metadata from `koruma-derive-core`
- mirrors validators into the holder type
- preserves shorthand and builder-chain validator forms
- injects required-value behavior where holder optionality would otherwise lose
  source-model required semantics

## Inventory Integration

When the `inventory` feature is enabled:

1. `GpuiForm` emits one `GpuiFormShape` per derived struct.
1. Each field becomes a `FieldVariant` with component contract metadata from
   `gpui-form-codegen`.
1. Metadata includes validation rule identifiers, defaults, full value type
   paths, component shape UI paths, and skipped-field information for
   downstream generators.

## Other Derives

### `ComponentShape`

- emits a `ComponentShape` impl directly for a state type
- defaults constructor wiring to `Self::new(window, cx)`
- accepts a constructor expression for `new = ...`; paths and closures receive
  `(window, cx)`, while direct constructor expressions are emitted as written
- optionally stores a component path for prototyping output
- optionally sets shape-level `VALUE_BINDING` metadata for
  `ComponentValueBinding<T>` prototyping hooks, including
  `seed_value_binding_state` and `form_value_change`; this is enabled with the
  bare `value_binding` flag and disabled by omitting it
- defaults to implementing `gpui_form_component::shape::ComponentShape`
  for downstream application crates
- optionally accepts `shape_crate = crate` for runtime crates that implement
  their local `shape::ComponentShape` path directly

### `component_shape!`

- emits a local zero-sized shape type plus `ComponentShape` impl
- accepts caller generics, where clauses, and outer attributes
- accepts the same `new`, `component`, `value_binding`, `field_suffix`, and
  `shape_crate` metadata keys as `#[derive(ComponentShape)]`
- accepts either semicolon or comma separators between metadata entries
- defaults omitted `new` metadata to `<State>::new(window, cx)`
- targets external component/state pairs that cannot directly implement
  `ComponentShape` because both the trait and state type are foreign
- emits the implementation against `gpui_form_component::shape` by default
- leaves optional `ComponentValueBinding<T>` implementations to the
  caller or collection crate

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
