# gpui-form-component-story

Storybook gallery for the runtime helpers in
[`gpui-form-component`](../gpui-form-component/README.md).

This package keeps demo UI and the launcher binary outside the runtime library
crate. Most users should depend on [`gpui-form`](../gpui-form/README.md) or
`gpui-form-component`, not this package.

Run the gallery with:

```sh
cargo run -p gpui-form-component-story
```

The launcher uses `gpui-form-component-story` as its stable local-preference
consumer namespace. GPUI Storybook resolves saved or system appearance, the
independent light and dark theme slots, language, and scrollbar intent before
the gallery window opens. Every resolved language is passed to this package's
anchored Fluent adapter as part of the same preference transition.

Story titles, descriptions, diagnostics, and other demo chrome are in-place
English strings. Text passed into the demo components is fluent-backed: the
`infinite_select` namespace covers demo enum metadata, while `date_picker` and
`file_picker` cover component placeholders, prompts, and action labels. Those
component-facing resources ship in English, French (`fr-FR`), and Simplified
Chinese (`zh-CN`).
