# gpui-form-prototyping-core

Generate GPUI form scaffolds from concrete `GpuiFormShape` inventory
registrations.

```toml
[dependencies]
gpui-form = { version = "0.6", features = ["inventory"] }
gpui-form-prototyping-core = "0.6"
```

Implement `FormLayout` for the target view, then adapt each registered shape:

```rust,ignore
use gpui_form::schema::registry::{GpuiFormShape, inventory};
use gpui_form_prototyping_core::FormShapeAdapter;

for shape in inventory::iter::<GpuiFormShape>() {
    FormShapeAdapter::new(shape).generate_file(&layout)?;
}
```

`FormShapeAdapter::parts()` exposes validated fragments for custom layouts,
while `generate_file(...)` renders a complete file. Generators that write into
another crate can remap source paths before rendering.

See [Prototyping](https://stayhydated.github.io/gpui-form/book/prototyping.html)
for the complete workflow and failure modes.
