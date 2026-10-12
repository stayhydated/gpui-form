# gpui-form-component

[![Codecov: gpui-form-component][codecov-badge]][codecov]
[![crates.io: gpui-form-component][crate-badge]][crate]

`gpui-form-component` provides reusable GPUI runtime components and form shapes
for localized date and file selection, cascading enum choices and controlled
ordered row editing.

## Overview

| Module | Purpose |
| --- | --- |
| `infinite_select` | Cascading selects over nested enum trees |
| `date_picker` | Localized single-date and date-range pickers |
| `file_picker` | Native file and directory selection |
| `row_editor` | Controlled typed rows with stable identity, insert, remove, duplicate and reorder |

The `derive` feature re-exports `#[derive(InfiniteSelect)]`. The
`component-shape` feature makes `InfiniteSelect<T>`, `DatePicker`,
`DateRangePicker`, `FilePicker`, and `RowEditor<Row, Config>` available in
`#[gpui_form(component(...))]` declarations.

`InfiniteSelectState` setters update the selection without emitting
`InfiniteSelectEvent`. Confirmed select changes report the previous and current
values and paths.

Initialize the application `gpui-es-fluent` resources before using localized
date, file, row editor or annotated infinite-select text. Render application
messages with `gpui_es_fluent::localize_message(cx, &message)` and type labels with
`gpui_es_fluent::localize_label::<MyType>(cx)`. An explicit localizer uses
`i18n.localize_message(&message)` and `MyType::localize_label(&i18n)`.

`RowEditorConfig<Row>` supplies stable nonempty IDs, creation/duplication and
field rendering. Subscribe to `RowEditorEvent<Row>` and validate its complete
requested collection before accepting it with `RowEditorState::set_rows`.
Programmatic updates preserve row identity and emit no change event. Apply the
row context's disabled and readonly flags to custom fields; the editor rejects
editing requests in both states. Use `can_insert` and `can_duplicate` to expose
caller-controlled availability consistently in controls and requested changes,
without allocating a row or invoking its factory.

[codecov-badge]: https://codecov.io/github/stayhydated/gpui-form/graph/badge.svg?branch=master&component=gpui-form-component
[codecov]: https://codecov.io/github/stayhydated/gpui-form
[crate-badge]: https://img.shields.io/crates/v/gpui-form-component.svg?label=gpui-form-component
[crate]: https://crates.io/crates/gpui-form-component
