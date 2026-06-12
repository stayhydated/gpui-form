# gpui-form-derive Architecture

`gpui-form-derive` owns the main proc-macro entry points for the workspace.

It parses user structs and enums, delegates component modeling to
`gpui-form-codegen`, and emits the generated types and inventory metadata that
power the rest of the ecosystem.

## Entry Points

`src/lib.rs` defines:

- `#[derive(GpuiForm)]`

The reusable GPUI component-shape runtime and primary shape declaration macros
live in `component-shape-gpui`.

`#[derive(InfiniteSelect)]` is not part of this crate. It lives in
`gpui-form-component-derive` and is re-exported by `gpui-form-component` when
that crate's `derive` feature is enabled.

`#[derive(SelectItem)]` is also not part of this crate. It lives in
`gpui-form-collection-derive`.

## Module Layout

- `src/derives/gpui_form/mod.rs`: `GpuiForm` entry module
- `src/derives/gpui_form/attrs.rs`: raw `#[gpui_form(...)]` field option
  grammar, struct marker attributes, and unknown/duplicate nested option errors
- `src/derives/gpui_form/intent.rs`: parsed field and struct intent before
  type normalization or holder/component planning
- `src/derives/gpui_form/ir.rs`: normalized semantic form/field IR shared by
  planning, value-holder emission, inventory emission, and type checks
- `src/derives/gpui_form/holder_plan.rs`: value-holder subplans for storage,
  defaults, holder conversion, validation, and present-field snapshots
- `src/derives/gpui_form/structs.rs`: macro entry options passed from
  `src/lib.rs`
- `src/derives/gpui_form/expansion.rs`: top-level `GpuiForm` expansion pipeline
- `src/derives/gpui_form/components.rs`: delegates component fields into
  codegen layouts
- `src/derives/gpui_form/value_holder.rs`: generated holder types, defaults,
  conversion logic, and skip-field handling
- `src/derives/gpui_form/mcp.rs`: generated MCP form descriptors, argument
  decoding, validation bridge, model-conversion bridge, and MCP submit
  registration
- `src/derives/gpui_form/validation.rs`: Koruma option parsing and projection
  into gpui-form validation metadata
- `src/derives/gpui_form/koruma.rs`: Koruma metadata mirroring helpers
- `src/derives/gpui_form/cfg_attr.rs`: `cfg_attr` flattening before parse-time
  inspection

## `GpuiForm` Expansion Pipeline

1. Parse the input with `syn`.
1. Flatten `cfg_attr` wrappers so downstream parsing sees effective
   `#[gpui_form(...)]` data.
1. Parse raw field options through `attrs.rs`, then assemble parsed field and
   struct intents in `intent.rs` with `darling`. Field-level component shapes
   are parsed with `component-shape-codegen` helpers into a typed base shape
   path and optional configured builder expression from inputs such as
   `#[gpui_form(component(my::Shape))]` or
   `#[gpui_form(component(my::Shape::configured(...)))]`; arbitrary Rust
   expressions are otherwise only parsed for intent-scoped `default`,
   `value(from_source = ...)`, and `value(into_source = ...)`. The parser
   rejects duplicate component expressions before codegen. Shape declarations
   own component metadata.
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
- optional MCP submit integration when the derive `mcp` feature is enabled and
  the form opts in with `#[gpui_form(mcp)]` (or `#[gpui_form(mcp(...))]`)

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
- skipped fields are prefilled when converting from the original model into the
  holder
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
  holder/source conversion, typed present-field snapshots, and required validation builder
  selection through a shared storage strategy instead of duplicating those
  decisions at each call site
- `ValueHolderPlan` collects per-field subplans before token rendering so
  generated holder code can consume semantic storage/default/conversion/
  validation/present-field facts instead of re-deriving broad field facts from
  `FieldPlan`. It also preserves skipped-field and original-field ordering for
  holder API emission, so `value_holder.rs` renders `present_fields()` and
  `into_original` from structured holder subplans.
- component fields cross the `gpui-form-codegen` boundary with a typed
  `ComponentShapeStoragePolicy` that can only represent shape-owned component
  storage. Derive planning combines that policy with source optionality once,
  while hidden fields select non-component direct storage in the derive layer.
  Inventory presence metadata reads the resulting `HolderStoragePlan`.
- value-holder emission receives the same `DeriveContext` as expansion, so
  holder-specific tokens reuse the already-resolved facade/runtime crate paths
  instead of resolving paths independently
- default and conversion expression lowering delegates to
  `gpui-form-codegen::metadata`, shared with prototyping generation. The derive
  supplies the original user-authored attribute spans for `default`,
  `from_source`, and `into_source` expressions so diagnostics stay anchored to
  the relevant field option.
- holder-to-model API shape is selected inside `ValueHolderPlan`, with
  `Infallible`, `FallibleRequired`, and `SkippedFields` modes. The holder plan
  stores semantic fallibility as `ConversionFallibility` and renders predicates
  only when emitting inventory metadata.
- holder-to-model conversion uses `TryFrom` when any non-skipped field can be
  missing without a default; `From` is reserved for holders the derive can prove
  infallible directly
- shape-policy holder conversion keeps the checked `try_into_original()` path
  when requiredness is inherited from the shape policy. A field-level default
  makes that field statically infallible and lets the normal infallible holder
  path apply.
- generated `FooFormValueHolder` is the public holder struct. Shape policy
  storage appears through associated type projections and where clauses.
- non-optional shape-backed fields whose `ValueStoragePolicy` can represent
  missing values also get generated Koruma validators so `validate()` reports
  missing shape-required values before holder-to-model conversion
- skipped-field holders expose a generated typed `PresentField` enum and
  `present_fields()` method for inspecting editable state. JSON/debug
  formatting lives in examples or prototyping output, not in every generated
  holder.

## Koruma Integration

`GpuiForm` can enable Koruma-aware holder generation even when the source
struct only contains field-level `#[koruma(...)]` attributes.

The derive layer:

- reads normalized validator metadata from `koruma-derive-core`
- mirrors validators into the holder type
- preserves direct validator chains, labels, target selectors, and `each(...)`
  element-validation groups
- injects required-value behavior where holder optionality would otherwise lose
  source-model required semantics

## Inventory Integration

When the `inventory` feature is enabled:

1. `GpuiForm` emits one `GpuiFormShape` per non-generic derived struct, and
   generic source structs must opt out with `#[gpui_form(no_inventory)]` when
   the `inventory` feature is enabled. Inventory items must name one concrete
   source type and module path; the generated form code carries the source
   generics.
1. Each non-skipped field becomes a `FieldVariant`. Component fields use
   `FieldVariant::component(...)` with component contract metadata from
   `gpui-form-codegen`; hidden fields use `FieldVariant::hidden(...)` with the
   same value/default/conversion/validation metadata and no component payload.
1. Metadata includes validation rule identifiers, defaults, full value type
   paths, component shape UI paths for component fields, and holder conversion
   metadata that records skipped-field reconstruction requirements for
   downstream generators.

## MCP Submit Integration

When the `mcp` feature is enabled, `#[derive(GpuiForm)]` still emits normal
`GpuiForm` output plus MCP integration only for forms that opt in with
`#[gpui_form(mcp)]` (or `#[gpui_form(mcp(...))]`).
The opt-in is rejected when the facade `mcp` feature is not enabled, when the
form also uses `no_inventory`, or when the source type is generic, because MCP
submit discovery registers concrete descriptor and handler functions in
inventory.
The generated MCP code is intentionally type-directed rather than
string-directed:

1. A hidden `McpField` slice is emitted from the same `FieldPlan` and
   `ValueHolderPlan` data used for holder and `GpuiFormShape` generation.
   Component-backed fields use `McpField::component::<Value, Shape>` so the
   generated descriptor is type-checked against both the field schema and the
   selected component shape metadata.
   Struct-level `#[gpui_form(mcp(name = ..., title = ..., description = ...))]`
   options are emitted as `McpToolMetadata` on the descriptor.
1. The source type implements `gpui_form::mcp::McpForm`; its
   `decode_arguments` method populates the generated `*FormValueHolder` from
   JSON object fields. Component-backed fields use the component shape's
   `ValueStorage::present` API and do not instantiate GPUI entities.
1. Koruma-enabled forms bridge generated holder `validate()` errors into MCP
   tool execution errors. Non-Koruma forms validate as a no-op at this layer.
1. Holders in `Infallible` and `FallibleRequired` conversion modes also
   implement `McpFormModel`, allowing `form::<Form>(&mut server).model(...)` to
   call an application-owned model handler. `SkippedFields` holders
   intentionally omit that model bridge; applications register them with holder
   handlers and supply skipped context themselves.
1. The source model, when model conversion exists, and the generated
   `*FormValueHolder` implement `McpSubmitArgument`. This lets
   `#[gpui_form::mcp_submit]` infer model versus holder registration from trait
   resolution instead of generated type-name suffixes.
1. A `gpui_form::mcp::registry::McpSubmitRegistration` inventory item is
   emitted for discovery.
1. The `#[gpui_form::mcp_submit]` attribute macro emits a separate
   `McpSubmitHandlerRegistration` inventory item for application-owned handler
   functions. It requires the handler argument to implement
   `McpSubmitArgument`, then routes synchronous and async handlers through that
   generated trait impl. Handlers must return `Result<T, E>`.
   The macro resolves the
   facade crate path with the same crate-path machinery used by derive output,
   propagates setup errors from registration, and does not synthesize handler
   behavior.

The derive does not emit CLI parsing, command help, shell completion, exit-code
policy, GPUI window creation, or submit-button dispatch for MCP.

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
