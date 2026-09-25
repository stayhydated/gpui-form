# gpui-form-component

[![Codecov: gpui-form-component][codecov-badge]][codecov]
[![crates.io: gpui-form-component][crate-badge]][crate]

`gpui-form-component` provides reusable GPUI runtime components and form shapes
for localized date and file selection and cascading enum choices.

## Overview

| Module | Purpose |
| --- | --- |
| `infinite_select` | Cascading selects over nested enum trees |
| `date_picker` | Localized single-date and date-range pickers |
| `file_picker` | Native file and directory selection |

The `derive` feature re-exports `#[derive(InfiniteSelect)]`. The
`component-shape` feature makes `InfiniteSelect<T>`, `DatePicker`,
`DateRangePicker`, and `FilePicker` available in
`#[gpui_form(component(...))]` declarations.

Initialize the application `gpui-es-fluent` resources before using localized
date, file, or annotated infinite-select text.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-component
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-component.svg?label=gpui-form-component
[crate]: https://crates.io/crates/gpui-form-component
