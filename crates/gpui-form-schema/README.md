# gpui-form-schema

Shared schema and inventory metadata for the `gpui-form` ecosystem.

Most applications should use [`gpui-form`](../gpui-form/README.md) instead.
Use this crate directly when you are building tooling, runtime integrations, or
prototyping flows around generated form metadata.

## What It Provides

- `registry::GpuiFormShape`
- `registry::{FieldValueSpec, FieldVariant}`
- `registry::{ComponentCapabilities, FieldComponentVariant, FieldValuePresence}`
- `registry::{RustType, RustPath, RustExpr, ComponentSuffix}`
- `registry::inventory`
- `resolved::{ResolvedComponentMetadata, ResolvedGpuiFormShape, ResolvedField}`

`GpuiFormShape::fields` is the inventory list of non-skipped form fields:
component-backed fields and explicit hidden value-holder-only fields.
`FieldVariant` records both source-model and form-side value types,
required-value holder behavior, conversion expressions, component shape
metadata, resolved component suffixes, opt-in value-binding metadata, and
field-facing label/description/example metadata for generators. Its fields are
private; construct complete value metadata with
`FieldValueSpec::new(value_type, source_value_type, value_presence)` before
calling `FieldVariant::component(field_name, value, component)` or
`FieldVariant::hidden(field_name, value)`. That keeps `FieldValuePresence`
explicit and prevents incomplete schema field construction. Component-only
metadata is built through
`FieldComponentVariant::new(shape_path)` before attaching it to a field, so
metadata cannot represent value-bound-without-shape. Internally it uses
`component_shape::ComponentShapeUse` for the backend-neutral field-to-shape
metadata while keeping form storage behavior in this crate. Helpers such as
`field_name_with_component_suffix()` expose the suffix-bearing prototyping name
used for DOM IDs, event handlers, and helper names, with a `"shape"` fallback
suffix. Generated component entity fields should use the source field name.
Component metadata groups render, value-binding, and storage behavior as
`ComponentCapabilities`.

Rust fragments in registry metadata use typed string wrappers so generators do
not accidentally pass a type where a path, expression, or component suffix is
expected. `RustType::new`, `RustPath::new`, and `RustExpr::new` validate the
fragment with `syn`; macro-generated inventory uses explicit
`from_macro_tokens_unchecked` const constructors because the derive layer
stringifies syntax it already parsed.
Use the wrapper `.parse()` methods when tooling needs the `syn` representation.
Generators that consume complete form metadata should prefer
`ResolvedGpuiFormShape`, which validates generated names, parses Rust syntax
once at the inventory boundary, and exposes typed `ResolvedField` /
`ResolvedComponentMetadata` facts for component paths, capabilities, value
presence, conversions, and validations. Treat direct `FieldVariant` string
accessors as the inventory serialization surface, not the normal generator API.
`ComponentSuffix` is re-exported from `gpui-form-core`, so schema inventory and
runtime prototyping metadata share the same suffix contract.

## Example

```rs
use gpui_form_schema::registry::{GpuiFormShape, inventory};

for shape in inventory::iter::<GpuiFormShape>() {
    println!("form: {}", shape.struct_name);

    for field in shape.fields {
        println!("  {} -> {}", field.field_name(), field.field_name_with_component_suffix());
    }
}
```

## When To Use This Crate Directly

- You are writing a generator that consumes `GpuiFormShape`
- You need runtime metadata about component shape contracts
- You want inventory access without depending on the facade crate

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for normal application development
- [`gpui-form-prototyping-core`](../gpui-form-prototyping-core/README.md) for
  scaffold generation over this metadata
