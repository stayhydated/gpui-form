# gpui-form-core

[![Codecov: gpui-form-core][codecov-badge]][codecov]
[![crates.io: gpui-form-core][crate-badge]][crate]

`gpui-form-core` provides UI-neutral form contracts and validation helpers for
lower-level integrations. Application crates normally use these APIs through
the [`gpui-form` facade][gpui-form].

## Overview

The public surface includes:

- `FormField`, implemented by generated typed field enums
- `ComponentSuffix` and suffix validation
- signed and unsigned numeric text validation

## Example

```rust
use gpui_form_core::numeric::validate_signed_numeric;

assert!(validate_signed_numeric::<i32>("-42", true));
```

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-core
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-core.svg?label=gpui-form-core
[crate]: https://crates.io/crates/gpui-form-core
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
