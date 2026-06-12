---
name: use-gpui-form-component-shapes
description: "Use when Codex needs to wire component-shape-gpui component shapes into gpui-form forms, including #[gpui_form(component(...))], gpui-form-runtime storage policy, generated form fields/components, value-holder behavior, or prototyping output."
---

# Use GPUI Form Component Shapes

## Scope Boundary

Use this skill for `gpui-form` integration work after a GPUI component shape
exists or needs to be selected. It covers how shapes participate in
`#[derive(gpui_form::GpuiForm)]`, generated form state, value storage, runtime
imports, and prototyping output.

Use `use-gpui-form` for ordinary generated forms that only use built-in,
collection, or component-owned shapes.

Use the component-shape repository skills for shape ownership and declaration:

- `use-component-shape` for framework-neutral shape metadata, suffixes,
  capabilities, and value-change concepts.
- `use-component-shape-gpui` for `component_shape_gpui::GpuiComponentShape`,
  `component_shape_gpui::component_shape!`, GPUI render contracts, and
  value-binding declarations.

This reusable skill does not cover proc-macro implementation internals,
repository maintenance, or contributor-only architecture work. Use the
workspace `AGENTS.md` and crate architecture docs for those tasks.

## Integration Rule

First decide whether the form can use an existing shape:

- Existing collection shape: prefer `gpui_form_collection::*` shapes when they
  already model the widget and value type.
- Existing component shape: use the shape directly in
  `#[gpui_form(component(...))]`.
- New custom shape: create or select the shape with the component-shape GPUI
  guidance first, then return here for gpui-form wiring.

Application crates that only derive forms normally depend on `gpui-form`.
Crates that define custom component-backed shapes should also depend on
`gpui-form-runtime` directly when they implement lower-level runtime policies
or value binding.

## Field Shape Usage

Use the shape type in the field attribute:

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(component(EmailInputShape))]
    pub email: String,
}
```

The shape must be compatible with the field value type through either explicit
shape metadata, a `GpuiComponentShapeFor<Value>` implementation, or a
`GpuiComponentValueBinding<Value>` implementation declared by the shape.

## Storage Policy

Reusable component shapes should implement `GpuiFormComponentShapePolicy` so
the generated form chooses the right value-holder strategy:

```rust
impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for EmailInputShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}
```

Use:

- `DirectValueStorage` when the component can synthesize a default value and
  non-optional fields should store direct `T` values.
- `RequiredValueStorage` when a required field should represent missing input
  as `Option<T>` in the generated holder.

When reviewing a field type mismatch, check the storage policy separately from
value compatibility. Compatibility determines whether a shape can support a
value type; storage policy determines how required values are represented.
When the shape should participate in MCP submit schemas, rely on inferred MCP
metadata for unambiguous declared values such as `String`, booleans, numbers,
dates, `Vec<T>`, set-like primitive collections, fixed arrays,
`gpui_form::mcp::McpRange<T>`, or `(Option<T>, Option<T>)` ranges. The
generated `ComponentShapeFor<Value>` impl carries the value-specific metadata,
so multi-value shapes can expose distinct MCP input shapes per supported
value. Component-backed form fields use value-specific shape MCP input metadata
when available, then fall back to the field type's `McpJsonSchema` metadata.
Use `McpJsonSchema` or a manual integration schema/decoder for ambiguous or
intentionally different wire shapes.

## Field-Level Builder Configuration

Use `gpui_form_runtime::shape::GpuiComponentShapeBuilder<Shape>` when a
reusable shape needs field-local construction options without creating a new
shape type for every variant. Expose direct shape helpers backed by Bon builder
setters, plus a `from(config)` helper for completed config values.

```rust
#[derive(gpui_form::bon::Builder)]
#[builder(crate = ::gpui_form::bon)]
pub struct SelectArgs {
    #[builder(default)]
    searchable: bool,
}

impl MySelect {
    pub fn searchable(searchable: bool) -> SelectArgs {
        SelectArgs::builder().searchable(searchable).build()
    }

    pub fn from(args: SelectArgs) -> SelectArgs {
        args
    }
}

impl gpui_form_runtime::shape::GpuiComponentShapeBuilder<MySelect> for SelectArgs {
    fn build(
        self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, <MySelect as gpui_form_runtime::shape::GpuiComponentShape>::State>,
    ) -> <MySelect as gpui_form_runtime::shape::GpuiComponentShape>::State {
        let mut state = MySelect::new(window, cx);
        if self.searchable {
            state = state.searchable(true);
        }
        state
    }
}
```

Then configure individual form fields:

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct SearchForm {
    #[gpui_form(component(MySelect::searchable(true)))]
    pub assignee: Option<String>,
}
```

The field still uses the same base shape for value compatibility, storage
policy, render metadata, and prototyping metadata; only state construction
changes.

## Prototyping Alignment

When a shape is used by inventory-driven prototyping, ensure it exposes stable
component prototyping metadata. In practice, custom shapes should set a
non-empty ASCII identifier suffix through component-shape metadata so generated
DOM IDs, event handlers, and helper names stay stable.

Generated `FormFields` members and `FormComponents` constructors use the source
field name. The component suffix affects generated helper naming around the
component role, not the original Rust field name.

## Documentation Sync

When changing user-visible form/component-shape behavior in this repository,
keep the relevant public surfaces aligned:

- root `README.md`
- `crates/gpui-form/README.md`
- affected crate READMEs
- `examples/README.md` and showcased examples when behavior changes
- in-repository `skills/*` guidance
- matching rustdocs, source-adjacent comments, tests, and examples when
  boundaries or behavior change

Do not duplicate generic component-shape macro guidance here; update the
component-shape-owned skills instead.
