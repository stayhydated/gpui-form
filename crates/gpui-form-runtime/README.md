# gpui-form-runtime

GPUI component-shape and value-storage contracts referenced by generated
`gpui-form` code.

Normal applications use these contracts through
`gpui_form::runtime::shape` and do not need a direct dependency. Depend on this
crate when defining reusable shapes, storage policy, or value binding in a
lower-level integration crate.

The public surface includes:

- re-exported `component-shape-gpui` construction, rendering, builder, and
  value-binding contracts
- `GpuiFormComponentShapePolicy`
- `DirectValueStorage` and `RequiredValueStorage`
- helpers for seeding component state and converting events to `ValueChange<T>`

See [Component shapes](https://stayhydated.github.io/gpui-form/book/component_shapes.html)
for integration guidance and [docs.rs](https://docs.rs/gpui-form-runtime/) for
the trait reference.
