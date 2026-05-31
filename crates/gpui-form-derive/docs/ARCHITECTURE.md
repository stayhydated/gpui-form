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
- `src/derives/component_shape.rs`: function-like component shape expansion for
  local wrappers around external component/state types

## `GpuiForm` Expansion Pipeline

1. Parse the input with `syn`.
1. Flatten `cfg_attr` wrappers so downstream parsing sees effective
   `#[gpui_form(...)]` data.
1. Parse struct-level and field-level `#[gpui_form(...)]` data with `darling`.
   Field-level component shapes are parsed into a typed shape path from
   `#[gpui_form(component(my::Shape))]`;
   arbitrary Rust expressions are only parsed for intent-scoped `default`,
   `value(from_source = ...)`, and `value(into_source = ...)`. The parser
   rejects duplicate component expressions before codegen and rejects
   field-level component metadata such as
   `component = ...` or `field_suffix = ...`; those belong on the shape
   declaration.
1. Parse Koruma field metadata through `koruma-derive-core`.
1. For each component field, delegate component-specific modeling to
   `gpui-form-codegen`, which resolves the component shape into a planned
   `ResolvedComponentShape` before layout, inventory, and type-check emission.
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
- `value(type = ..., from_source = ..., into_source = ...)` conversions
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
- parsed field intent carries `FieldContext` and `FieldAttrContext` forward
  into planning, so component planning and value conversion assertions can
  anchor generated diagnostics to the relevant field or `gpui_form` option
  span instead of recomputing spans later
- structured field syntax nests value conversion/default metadata inside the
  component or hidden intent: `component(Shape, value(...), default = ...)` and
  `hidden(value(...), default = ...)`. Flat top-level `type`, `source_to_form`,
  `form_to_source`, and `default` field options are rejected during parsing.
- hidden fields are explicit value-holder-only fields and may use default and
  conversion metadata
- skipped fields reject component and hidden intent during field parsing
- expansion builds one `FormPlan` after component field generation; inventory
  metadata, value-holder generation, type checks, validation wiring, defaults,
  conversions, and shape requiredness all read from that plan instead of
  recomputing field type facts independently
- skipped fields are split from rendered `HolderFieldIr` facts before value
  holder emission, so holder storage, form type, validation, default, and
  conversion code paths do not need defensive "maybe skipped" lookups
- rendered field storage is represented as `HolderStoragePlan`; value-holder
  emission routes storage type, present-value wrapping, default storage,
  holder/source conversion, present-field JSON, and required validation builder
  selection through a shared storage strategy instead of duplicating those
  decisions at each call site
- value-holder emission receives the same `DeriveContext` as expansion, so
  holder-specific tokens reuse the already-resolved facade/runtime crate paths
  instead of resolving paths independently
- holder-to-model API shape is selected by `HolderConversionPlan`, with
  `Infallible`, `FallibleRequired`, and `SkippedFields` modes. The plan stores
  semantic fallibility as `ConversionFallibility` and renders predicates only
  when emitting inventory metadata.
- holder-to-model conversion uses `TryFrom` when any non-skipped field can be
  missing without a default; `From` is reserved for holders the derive can prove
  infallible directly
- shape-policy holder conversion keeps the checked `try_into_original()` path
  when requiredness is inherited from the shape policy. A field-level default
  makes that field statically infallible and lets the normal infallible holder
  path apply.
- generated `FooFormValueHolder` is the public holder struct. Shape policy
  storage appears through associated type projections and where clauses rather
  than a hidden storage struct plus public alias.
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
- parses shape metadata through the shared `ShapeOption` option model used by
  `component_shape!`
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
- parses shape metadata through the shared `ShapeOption` option model used by
  `#[derive(ComponentShape)]`
- uses semicolon separators between metadata entries
- accepts nested `impl` items and emits them after the generated
  `ComponentShape` impl
- sets shape-level `ValueBindingPolicy` only when `value_binding;` is present
- defaults omitted `new` metadata to `<State>::new(window, cx)`
- emits one `ComponentShapeFor<T>` impl per `value = ...` / `values(...)`
  metadata entry
- rejects duplicate value metadata and rejects mixing value metadata with
  manual `ComponentShapeFor` impls in the same block
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
