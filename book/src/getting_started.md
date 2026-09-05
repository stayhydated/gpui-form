# Getting started

This tutorial adds a typed form model to an existing GPUI application. At the
end, `cargo check` recognizes the generated `UserProfileForm*` types and the
application owns the holder and component entities needed to render the form.

## Prerequisites

- Rust 1.98 or newer.
- A GPUI application that calls `gpui_kit::init(cx)`.
- A `gpui_kit::component::Root` around each first-level window view.
- `gpui-kit` 0.6.0.

## Add the dependencies

Use `gpui-form` for the derive and generated runtime paths. This example uses
ready-made collection shapes and the collection select derive:

```toml
[dependencies]
gpui-kit = "0.6.0"
gpui-form = "0.6"
gpui-form-collection = "0.6"
gpui-form-collection-derive = "0.6"
strum = { version = "0.28", features = ["derive"] }
```

## Derive the form

Give every field one `component(...)`, `hidden`, or `skip` intent:

```rust,ignore
use gpui_form::GpuiForm;
use gpui_form_collection_derive::SelectItem;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, PartialEq, SelectItem)]
enum Country {
    #[default]
    UnitedStates,
    France,
    Japan,
}

#[derive(Clone, Debug, Default, GpuiForm)]
struct UserProfile {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    username: Option<String>,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    age: Option<u32>,

    #[gpui_form(component(
        gpui_form_collection::select::Select::<_>,
        default = Country::France
    ))]
    country: Country,

    #[gpui_form(component(gpui_form_collection::checkbox::Checkbox))]
    subscribe: bool,
}
```

The default belongs to the component intent and seeds the generated holder.
Optional fields start as `None`. Direct-storage fields without an explicit
default use the form-side type's `Default` implementation.

## Own the generated state

Create the holder and component entities in the GPUI entity that renders the
form:

```rust,ignore
let holder = UserProfileFormValueHolder::default();
let username = cx.new(|cx| UserProfileFormComponents::username(window, cx));
let age = cx.new(|cx| UserProfileFormComponents::age(window, cx));
let country = cx.new(|cx| UserProfileFormComponents::country(window, cx));
let subscribe = cx.new(|cx| UserProfileFormComponents::subscribe(window, cx));

let fields = UserProfileFormFields {
    username,
    age,
    country,
    subscribe,
};
```

The constructors create widget state. A complete view must also:

1. Retain each `gpui_kit::Subscription` on the owning GPUI entity.
2. Map component events through
   `gpui_form::runtime::shape::value_change` into the holder.
3. Seed widget state with
   `gpui_form::runtime::shape::seed_value_binding_state`.
4. Render each shape's generated render component.
5. Validate and convert the holder before invoking application submission code.

The inventory workflow in [Prototyping](prototyping.md) generates this wiring
from the same shape metadata.

## Choose optional features

| Need | Configuration |
|---|---|
| Inventory metadata | `gpui-form = { version = "0.6", features = ["inventory"] }` |
| MCP tools in a GPUI application | `gpui-form = { version = "0.6", features = ["mcp"] }` |
| Headless MCP forms | `gpui-form = { version = "0.6", default-features = false, features = ["derive", "mcp"] }` |
| MCP schemas for Chrono or decimal values | Add `chrono` or `rust_decimal` beside `mcp` |
| Localized date, file, or infinite-select shapes | Add `gpui-form-component` with `component-shape` and, for the derive, `derive` |

## Check the result

Run:

```sh
cargo check
```

A successful check confirms that the dependencies resolve, each field has an
intent, and every selected shape supports its form-side value type.

## Troubleshooting

| Symptom | Action |
|---|---|
| `field ... must choose a gpui_form field intent` | Add `component(...)`, `hidden`, or `skip` to the named field. |
| A `gpui_form_component` shape is unavailable | Enable that crate's `component-shape` feature; infinite-select enums also need its `derive` feature. |
| GPUI types from two dependencies do not match | Depend on `gpui-kit` alone, as shown above. |
| The widget renders but the holder never changes | Retain the subscription and map its event into the holder, or generate the wiring through the prototyping workflow. |
