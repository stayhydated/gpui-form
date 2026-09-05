use gpui_kit::component::form::{Field, field};
use gpui_kit::{IntoElement, ParentElement as _, SharedString, Styled as _, div, px};

pub(super) fn story_panel(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    content: impl IntoElement,
) -> impl IntoElement {
    let title = title.into();
    let description = description.into();

    div()
        .max_w(px(620.))
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_lg().child(title))
                .child(div().text_sm().child(description)),
        )
        .child(content)
}

pub(super) fn story_field(
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    content: impl IntoElement,
) -> Field {
    let label = label.into();
    let description = description.into();

    field()
        .label(label)
        .description_fn(move |_, _| {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().child(description.clone()))
        })
        .child(content)
}
