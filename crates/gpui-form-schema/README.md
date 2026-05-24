# gpui-form-schema

Shared schema and inventory metadata for the `gpui-form` ecosystem.

Most applications should use [`gpui-form`](../gpui-form/README.md) instead.
Use this crate directly when you are building tooling, runtime integrations, or
prototyping flows around generated form metadata.

## What It Provides

- `components::ComponentKind`
- `components::ComponentsBehaviour`
- `registry::GpuiFormShape`
- `registry::FieldVariant`
- `registry::inventory`

`FieldVariant` records both source-model and form-side value types,
required-value holder behavior, conversion expressions, component shape
paths, component shape prototyping suffixes, and opt-in value-binding
metadata for generators. Helpers such as `field_name_with_behaviour()` keep
inventory consumers aligned with the field names emitted by
`#[derive(GpuiForm)]`.

## Example

```rs
use gpui_form_schema::registry::{GpuiFormShape, inventory};

for shape in inventory::iter::<GpuiFormShape>() {
    println!("form: {}", shape.struct_name);

    for field in shape.components {
        println!("  {} -> {}", field.field_name, field.behaviour.component_name());
    }
}
```

## When To Use This Crate Directly

- You are writing a generator that consumes `GpuiFormShape`
- You need runtime metadata about component shape behavior
- You want inventory access without depending on the facade crate

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for normal application development
- [`gpui-form-prototyping-core`](../gpui-form-prototyping-core/README.md) for
  scaffold generation over this metadata
