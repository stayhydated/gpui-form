# gpui-form-runtime

Runtime contracts referenced by `gpui-form` generated code.

Most applications start with [`gpui-form`](../gpui-form/README.md) and add
`gpui-form-runtime` when they use component-backed fields, app-owned
`ComponentShape`s, or value-bound component prototyping.

```toml
[dependencies]
gpui-form = { version = "*", features = ["derive"] }
gpui-form-runtime = "*"
gpui-form-derive = "*"
```

## Component Shapes

`shape::ComponentShape` is the runtime trait implemented by component shapes.
The derive macro stores `Entity<<Shape as ComponentShape>::State>` in generated
form fields and calls `ComponentShape::new(window, cx)` from the generated
component constructor.

For owned component state, derive the impl:

```rust
use gpui_form_derive::ComponentShape;

#[derive(ComponentShape)]
#[gpui_form_shape(field_suffix = "input")]
pub struct EmailInputState;
```

For wrapper shapes around state from another crate, use the proc macro:

```rust
gpui_form_derive::component_shape! {
    pub struct EmailInputShape {
        type State = gpui_component::input::InputState;
        new = gpui_component::input::InputState::new;
        field_suffix = "input";
    }
}
```

## Value Binding

Implement `shape::ComponentValueBinding<T>` when generated prototyping code
should seed a component from a form value and map component events back into the
form value holder.
