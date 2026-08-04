# gpui-form-core

UI-neutral contracts and validation helpers shared by `gpui-form`.

Application crates normally use the
[`gpui-form` facade](../gpui-form/README.md). Depend on this crate directly when
you need these helpers without the GPUI runtime:

- `FormField`, implemented by generated typed field enums
- `ComponentSuffix` and suffix validation
- signed and unsigned numeric text validation

```rust
use gpui_form_core::numeric::validate_signed_numeric;

assert!(validate_signed_numeric::<i32>("-42", true));
```

See the [API documentation](https://docs.rs/gpui-form-core/) for the complete
surface.
