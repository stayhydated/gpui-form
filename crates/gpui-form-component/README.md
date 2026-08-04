# gpui-form-component

Reusable GPUI runtime components and form shapes that complement
`gpui-form-collection`.

```toml
[dependencies]
gpui-form-component = { version = "0.5", features = ["component-shape", "derive"] }
```

## Modules

| Module | Purpose |
|---|---|
| `infinite_select` | Cascading selects over nested enum trees |
| `date_picker` | Localized single-date and date-range pickers |
| `file_picker` | Native file and directory selection |

The `derive` feature re-exports `#[derive(InfiniteSelect)]`. The
`component-shape` feature makes `InfiniteSelect<T>`, `DatePicker`,
`DateRangePicker`, and `FilePicker` available directly in
`#[gpui_form(component(...))]`.

```rust
#[gpui_form(component(
    gpui_form_component::infinite_select::InfiniteSelect::<_>.searchable(true)
))]
pub location: Location;
```

Initialize the application `gpui-es-fluent` resources before using localized
date, file, or annotated infinite-select text.

See [Component shapes](https://stayhydated.github.io/gpui-form/book/component_shapes.html)
for form integration. The runtime APIs are documented on
[docs.rs](https://docs.rs/gpui-form-component/).
