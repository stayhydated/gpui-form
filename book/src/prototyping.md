# Prototyping

Inventory prototyping turns concrete `GpuiForm` registrations into GPUI form
scaffolds. A generator should discover the expected form types, write the
configured output files, and format them with `rustfmt`; the workspace example
performs all three steps.

## Enable inventory

Enable the `inventory` feature in the dependency graph that contains the form
models:

```toml
[dependencies]
gpui-form = { version = "0.5", features = ["inventory"] }
gpui-form-prototyping-core = "0.5"
```

The registry describes each concrete form, its field intents, component shapes,
storage policy, rendering capability, value binding, holder conversion, and
stable prototyping suffix.

The generator binary must link the crate that owns the forms so its inventory
registrations are present. In this example, replace `app_models` with that
crate's Rust import name. Inspect the registry before writing files:

```rust,ignore
use gpui_form::schema::registry::{GpuiFormShape, inventory};

extern crate app_models;

for shape in inventory::iter::<GpuiFormShape>() {
    println!("{}: {} fields", shape.struct_name, shape.fields.len());
}
```

Seeing the expected struct names is the first verification checkpoint. An empty
iterator usually means the generator does not link the model crate or the
`inventory` feature is absent from that build graph.

## Generate scaffolds

Implement `gpui_form_prototyping_core::FormLayout` for the application's target
view, then pass each registration to
`gpui_form_prototyping_core::FormShapeAdapter::new(shape).generate_file(&layout)`.
The adapter produces the component construction, subscriptions, holder seeding,
rendering, validation, and conversion fragments used by the layout.

The workspace example is a complete generator. From this repository root, run:

```sh
cargo run -p prototyping
```

The command rewrites Rust scaffolds in
`examples/some-lib-forms/src/forms` and mirrors them under
`examples/prototyping/output`, then runs `rustfmt`. Success ends with
`Form generation complete.` for each output directory.

## Keep registrations concrete

Generic form structs should use `#[gpui_form(no_inventory)]`; inventory entries
must describe a concrete form type.

Generated scaffolds are consumers of component-shape metadata. When generation
reports missing render, value-binding, shape-path, or storage information,
update the owning shape declaration and regenerate.

## Troubleshooting

| Symptom | Action |
|---|---|
| No forms are discovered | Link the form-owning crate in the generator and enable `gpui-form/inventory` in the same dependency graph. |
| A generic form fails inventory registration | Add `#[gpui_form(no_inventory)]` to that generic form and register concrete form types instead. |
| Generation reports incomplete component capabilities | Add the missing render, value-binding, shape path, or storage metadata to the owning shape. The derive uses a generic `shape` suffix when the shape does not publish one. |
| Generation fails while formatting | Install the `rustfmt` component and rerun the generator. |
