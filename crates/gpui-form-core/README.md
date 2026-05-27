# gpui-form-core

UI-neutral helper logic for the `gpui-form` ecosystem.

Most application code should use [`gpui-form`](../gpui-form/README.md) instead.
Reach for this crate directly only when you want the helper logic without the
GPUI runtime layer.

## What It Provides

Today this crate is intentionally small and focused:

- `numeric::validate_signed_numeric`
- `numeric::validate_unsigned_numeric`

These helpers match the text-entry rules used by `gpui-form` number inputs.

## Example

```rs
use gpui_form_core::numeric::{
    validate_signed_numeric,
    validate_unsigned_numeric,
};

assert!(validate_signed_numeric::<i32>("-42", true));
assert!(validate_unsigned_numeric::<u32>("42", true));

assert!(!validate_signed_numeric::<i32>("01", true));
assert!(!validate_unsigned_numeric::<u32>("-1", true));
```

## When To Use This Crate Directly

- You are building your own numeric input wrapper and want the same text-entry
  rules as `gpui-form`
- You want the validation helpers without pulling in `gpui` or
  `gpui-component`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for normal application development
- [`gpui-form-runtime`](../gpui-form-runtime/README.md) when you need the
  component shape runtime contracts
- [`gpui-form-component`](../gpui-form-component/README.md) when you need the
  GPUI runtime helpers
