use gpui_form_component::row_editor::{
    RowEditor, RowEditorConfig, RowEditorEvent, RowEditorRowContext, RowEditorState,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _,
    Render, SharedString, Styled as _, Subscription, Window,
    component::{Disableable as _, button::Button},
    div,
};

use super::common::story_panel;

#[derive(Clone)]
struct CounterRow {
    id: u64,
    count: u64,
}

struct CounterFields {
    next_id: u64,
}

impl Default for CounterFields {
    fn default() -> Self {
        Self { next_id: 3 }
    }
}

impl RowEditorConfig<CounterRow> for CounterFields {
    fn row_id(&self, row: &CounterRow) -> SharedString {
        row.id.to_string().into()
    }

    fn create_row(&mut self) -> Option<CounterRow> {
        let id = self.next_id;
        self.next_id += 1;
        Some(CounterRow { id, count: 0 })
    }

    fn can_insert(&self, rows: &[CounterRow]) -> bool {
        rows.len() < 5
    }

    fn duplicate_row(&mut self, source: &CounterRow) -> Option<CounterRow> {
        let mut row = self.create_row()?;
        row.count = source.count;
        Some(row)
    }

    fn can_duplicate(&self, _: &CounterRow, rows: &[CounterRow]) -> bool {
        self.can_insert(rows)
    }

    fn row_label(&self, row: &CounterRow) -> SharedString {
        format!("Counter {}", row.id).into()
    }

    fn render_row(
        &self,
        row: &CounterRow,
        context: RowEditorRowContext<CounterRow, Self>,
        _: &mut Window,
        _: &mut App,
    ) -> AnyElement {
        let editor = context.editor().clone();
        let id = context.id().clone();
        let next = CounterRow {
            count: row.count + 1,
            ..row.clone()
        };
        // This field renders accepted values directly; rejected requests cannot
        // leave an independently retained field displaying an unaccepted value.
        Button::new("increment")
            .label(format!("Count: {} (+1)", row.count))
            .disabled(context.is_disabled() || context.is_readonly())
            .on_click(move |_, _, cx| {
                editor.update(cx, |state, cx| {
                    let _ = state.request_replace(&id, next.clone(), cx);
                });
            })
            .into_any_element()
    }
}

#[gpui_storybook::story]
#[derive(gpui_storybook::StoryControls)]
pub struct RowEditorStory {
    editor: Entity<RowEditorState<CounterRow, CounterFields>>,
    last_change: SharedString,
    _subscription: Subscription,
}

impl gpui_storybook::Story for RowEditorStory {
    fn title(_: &App) -> String {
        "Row Editor".into()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for RowEditorStory {
    fn focus_handle(&self, cx: &App) -> gpui_kit::FocusHandle {
        self.editor.focus_handle(cx)
    }
}

impl RowEditorStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            let mut state = RowEditorState::new(window, cx);
            state
                .set_rows(
                    vec![
                        CounterRow { id: 1, count: 1 },
                        CounterRow { id: 2, count: 2 },
                    ],
                    window,
                    cx,
                )
                .expect("initial counter IDs are unique");
            state
        });
        let subscription = cx.subscribe_in(
            &editor,
            window,
            |this: &mut Self, editor, event: &RowEditorEvent<CounterRow>, window, cx| {
                if event.rows().iter().all(|row| row.count <= 9) {
                    editor
                        .update(cx, |state, cx| {
                            state.set_rows(event.rows().to_vec(), window, cx)
                        })
                        .expect("editor requests preserve counter identities");
                    this.last_change = format!("Accepted {:?}", event.change()).into();
                } else {
                    this.last_change = "Rejected: a counter must be at most 9.".into();
                }
                cx.notify();
            },
        );
        Self {
            editor,
            last_change: "Edit, duplicate, and reorder up to five counters.".into(),
            _subscription: subscription,
        }
    }
}

impl Render for RowEditorStory {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        story_panel(
            "Ordered counters",
            "The caller validates each complete request. Counts above 9 are rejected without changing accepted rows.",
            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(RowEditor::new(&self.editor))
                .child(self.last_change.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use gpui_kit::{TestAppContext, component::Root, px, size, test::TestWindowExt as _};

    use super::*;

    #[gpui_kit::test]
    fn gallery_rejects_an_invalid_field_edit_and_limits_insertions(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        cx.update(|cx| {
            gpui_es_fluent::init_with_language(cx, unic_langid::langid!("en")).unwrap();
        });
        let mut story = None;
        let handle = cx.open_window(size(px(700.), px(750.)), |window, cx| {
            let view = cx.new(|cx| RowEditorStory::new(window, cx));
            story = Some(view.clone());
            Root::new(view, window, cx)
        });
        let story = story.unwrap();
        for _ in 0..9 {
            cx.update_window(handle.into(), |_, window, cx| {
                window.render_frame(cx);
                window.within("1").click("increment", cx);
            })
            .unwrap();
            cx.run_until_parked();
        }
        cx.update_window(handle.into(), |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(
                window.within("1").find("increment").label(),
                Some("Count: 9 (+1)")
            );
            assert_eq!(story.read(cx).editor.read(cx).rows()[0].count, 9);
            assert_eq!(
                story.read(cx).last_change.as_ref(),
                "Rejected: a counter must be at most 9."
            );
        })
        .unwrap();
        for _ in 0..4 {
            cx.update_window(handle.into(), |_, window, cx| {
                window.render_frame(cx);
                window.click("insert-row", cx);
            })
            .unwrap();
            cx.run_until_parked();
        }
        cx.update(|cx| {
            let editor = story.read(cx).editor.read(cx);
            assert_eq!(editor.rows().len(), 5);
            assert!(!editor.can_insert());
            assert!(!editor.can_duplicate("1"));
        });
    }
}
