# gpui-form-component

[![Codecov: gpui-form-component][codecov-badge]][codecov]
[![crates.io: gpui-form-component][crate-badge]][crate]

`gpui-form-component` provides reusable GPUI runtime components and form shapes
for localized date and file selection and cascading enum choices. Depend on
it directly for widgets; use it alongside the [`gpui-form` facade][gpui-form]
with `component-shape` enabled for derived forms.

Requires Rust 1.99 or newer and GPUI Kit 0.8.0. Initialize the application
`gpui-es-fluent` resources before rendering localized component text.

## Overview

| Module | Purpose |
| --- | --- |
| `infinite_select` | Cascading selects over nested enum trees |
| `date_picker` | Localized single-date and date-range pickers |
| `file_picker` | Native file and directory selection |

## Features

No features are enabled by default.

| Feature | Default | Purpose |
| --- | --- | --- |
| `component-shape` | No | Integrate `InfiniteSelect<T>`, `DatePicker`, `DateRangePicker`, and `FilePicker` with derived forms. |
| `derive` | No | Re-export `#[derive(InfiniteSelect)]` for nested enum trees. |

The [API reference][api] owns component initialization, events, and state
semantics. The [component gallery][gallery] demonstrates localized widgets.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-component
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-component.svg?label=gpui-form-component
[crate]: https://crates.io/crates/gpui-form-component
[gpui-form]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form/README.md
[api]: https://docs.rs/gpui-form-component/latest/gpui_form_component/
[gallery]: https://github.com/stayhydated/gpui-form/blob/master/crates/gpui-form-component-story/README.md
