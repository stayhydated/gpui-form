use std::{assert_matches, cell::Cell, rc::Rc};

use gpui_form_component::row_editor::{
    RowEditor, RowEditorChange, RowEditorConfig, RowEditorEvent, RowEditorIdentityError,
    RowEditorRowContext, RowEditorState,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, TestAppContext, Window,
    component::{
        Root,
        input::{Input, InputEvent, InputState},
    },
    div, px, size,
    test::TestWindowExt as _,
};

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct Record {
    id: String,
    name: String,
}

#[derive(Clone)]
pub struct Fields {
    next: Rc<Cell<u64>>,
    insert_available: Rc<Cell<bool>>,
    duplicate_available: Rc<Cell<bool>>,
}
impl Default for Fields {
    fn default() -> Self {
        Self {
            next: Rc::new(Cell::new(100)),
            insert_available: Rc::new(Cell::new(true)),
            duplicate_available: Rc::new(Cell::new(true)),
        }
    }
}
impl Fields {
    fn fresh(&self, name: String) -> Record {
        let next = self.next.get();
        self.next.set(next + 1);
        Record {
            id: format!("row-{next}"),
            name,
        }
    }
}
struct NameField {
    input: Entity<InputState>,
    accepted: String,
    _subscription: Subscription,
}
impl RowEditorConfig<Record> for Fields {
    fn row_id(&self, row: &Record) -> SharedString {
        row.id.clone().into()
    }
    fn row_label(&self, row: &Record) -> SharedString {
        format!("{}: {}", row.id, row.name).into()
    }
    fn create_row(&mut self) -> Option<Record> {
        Some(self.fresh("New".into()))
    }
    fn can_insert(&self, _: &[Record]) -> bool {
        self.insert_available.get()
    }
    fn duplicate_row(&mut self, row: &Record) -> Option<Record> {
        Some(self.fresh(row.name.clone()))
    }
    fn can_duplicate(&self, row: &Record, _: &[Record]) -> bool {
        self.duplicate_available.get() && row.name != "Locked"
    }
    fn render_row(
        &self,
        row: &Record,
        context: RowEditorRowContext<Record, Self>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let id = row.id.clone();
        let name = row.name.clone();
        let editor = context.editor().downgrade();
        let field = window.use_keyed_state(
            SharedString::from(format!("{}/{id}/name", context.editor().entity_id())),
            cx,
            |window, cx| {
                let input = cx.new(|cx| InputState::new(window, cx).default_value(name));
                let subscription = cx.subscribe_in(
                    &input,
                    window,
                    move |_: &mut NameField, input, event, _, cx| {
                        if !matches!(event, InputEvent::Change) {
                            return;
                        }
                        let name = input.read(cx).value().to_string();
                        let _ = editor.update(cx, |state, cx| {
                            if let Some(row) = state.rows().iter().find(|row| row.id == id).cloned()
                                && row.name != name
                            {
                                let _ = state.request_replace(&id, Record { name, ..row }, cx);
                            }
                        });
                    },
                );
                NameField {
                    accepted: row.name.clone(),
                    input,
                    _subscription: subscription,
                }
            },
        );
        if field.read(cx).accepted != row.name {
            field.update(cx, |field, cx| {
                field.accepted = row.name.clone();
                if field.input.read(cx).value().as_ref() != row.name {
                    field.input.update(cx, |input, cx| {
                        input.set_value(row.name.clone(), window, cx)
                    });
                }
            });
        }
        Input::new(&field.read(cx).input)
            .id("name")
            .disabled(context.is_disabled())
            .readonly(context.is_readonly())
            .into_any_element()
    }
}

#[cfg(feature = "component-shape")]
#[derive(Clone, Debug, Eq, gpui_form::GpuiForm, PartialEq)]
struct CollectionForm {
    #[gpui_form(component(RowEditor::<Record, Fields>, default = Vec::new()))]
    rows: Vec<Record>,
}

#[cfg(feature = "component-shape")]
#[derive(Clone, Debug, Eq, gpui_form::GpuiForm, PartialEq)]
struct OptionalCollection {
    #[gpui_form(component(RowEditor::<Record, Fields>))]
    rows: Option<Vec<Record>>,
}

struct Collection {
    editor: Entity<RowEditorState<Record, Fields>>,
    events: Vec<RowEditorEvent<Record>>,
    accept: bool,
    #[cfg(feature = "component-shape")]
    holder: CollectionFormFormValueHolder,
    _subscription: Subscription,
}
impl Collection {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::with_config(None, window, cx)
    }
    fn with_config(config: Option<Fields>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = if let Some(config) = config {
            cx.new(|cx| RowEditorState::with_config(config, cx))
        } else {
            #[cfg(feature = "component-shape")]
            let editor = cx.new(|cx| {
                <RowEditor<Record, Fields> as gpui_form::runtime::shape::GpuiComponentShape>::new(
                    window, cx,
                )
            });
            #[cfg(not(feature = "component-shape"))]
            let editor = cx.new(|cx| RowEditorState::new(window, cx));
            editor
        };
        let rows = vec![
            Record {
                id: "row-a".into(),
                name: "Alpha".into(),
            },
            Record {
                id: "row-b".into(),
                name: "Beta".into(),
            },
        ];
        editor.update(cx, |state, cx| {
            state.set_rows(rows.clone(), window, cx).unwrap()
        });
        let subscription = cx.subscribe_in(
            &editor,
            window,
            |this: &mut Self, editor, event, window, cx| {
                this.events.push(event.clone());
                if this.accept {
                    #[cfg(feature = "component-shape")]
                    {
                        use gpui_form::runtime::shape::{
                            ValueChange, seed_value_binding_state, value_change,
                        };
                        match value_change::<RowEditor<Record, Fields>, Vec<Record>>(
                            editor.read(cx),
                            event,
                        ) {
                            ValueChange::Set(rows) => this.holder.rows = rows,
                            _ => panic!("collection edits must carry a present typed collection"),
                        }
                        editor.update(cx, |state, cx| {
                            seed_value_binding_state::<RowEditor<Record, Fields>, Vec<Record>>(
                                state,
                                Some(&this.holder.rows),
                                window,
                                cx,
                            )
                        });
                    }
                    #[cfg(not(feature = "component-shape"))]
                    editor.update(cx, |state, cx| {
                        state.set_rows(event.rows().to_vec(), window, cx).unwrap()
                    });
                }
                cx.notify();
            },
        );
        Self {
            editor,
            events: Vec::new(),
            accept: true,
            #[cfg(feature = "component-shape")]
            holder: CollectionFormFormValueHolder::from(CollectionForm { rows }),
            _subscription: subscription,
        }
    }
}
impl Render for Collection {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_4()
            .child(RowEditor::new(&self.editor).id("editor"))
    }
}
fn open(cx: &mut TestAppContext) -> (gpui_kit::WindowHandle<Root>, Entity<Collection>) {
    cx.update(gpui_kit::init);
    cx.update(|cx| {
        gpui_es_fluent::init_with_language(cx, unic_langid::langid!("en")).unwrap();
    });
    let mut collection = None;
    let handle = cx.open_window(size(px(700.), px(650.)), |window, cx| {
        let view = cx.new(|cx| Collection::new(window, cx));
        collection = Some(view.clone());
        Root::new(view, window, cx)
    });
    (handle, collection.unwrap())
}
fn ids(collection: &Entity<Collection>, cx: &App) -> Vec<String> {
    collection
        .read(cx)
        .editor
        .read(cx)
        .rows()
        .iter()
        .map(|row| row.id.clone())
        .collect()
}

#[gpui_kit::test]
fn caller_availability_blocks_pointer_keyboard_and_imperative_factories(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    cx.update(|cx| gpui_es_fluent::init_with_language(cx, unic_langid::langid!("en")).unwrap());
    let config = Fields::default();
    config.insert_available.set(false);
    config.duplicate_available.set(false);
    let mut collection = None;
    let handle = cx.open_window(size(px(700.), px(650.)), |window, cx| {
        let view = cx.new(|cx| Collection::with_config(Some(config.clone()), window, cx));
        collection = Some(view.clone());
        Root::new(view, window, cx)
    });
    let collection = collection.unwrap();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window
            .within("editor")
            .within("row-a")
            .click("select-row", cx);
        window.within("editor").click("insert-row", cx);
        window.press("insert", cx);
        window.press("secondary-d", cx);
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| {
            assert!(!state.can_insert());
            assert!(!state.can_duplicate("row-a"));
            state.request_insert(None, cx).unwrap();
            state.request_duplicate("row-a", cx).unwrap();
        });
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        assert_eq!(ids(&collection, cx), ["row-a", "row-b"]);
        assert!(collection.read(cx).events.is_empty());
        assert_eq!(
            config.next.get(),
            100,
            "availability must not probe a factory"
        );
        config.insert_available.set(true);
        config.duplicate_available.set(true);
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |_, cx| cx.notify());
        window.render_frame(cx);
        window.within("editor").click("insert-row", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(ids(&collection, cx), ["row-a", "row-100", "row-b"]);
        window.press("secondary-d", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        assert_eq!(collection.read(cx).events.len(), 2);
        assert_eq!(config.next.get(), 102);
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| {
            state
                .set_rows(
                    vec![
                        Record {
                            id: "row-a".into(),
                            name: "Alpha".into(),
                        },
                        Record {
                            id: "row-b".into(),
                            name: "Locked".into(),
                        },
                    ],
                    window,
                    cx,
                )
                .unwrap();
            state.select_row("row-b", window, cx);
            assert!(!state.can_duplicate("row-b"));
        });
        window.render_frame(cx);
        window.press("secondary-d", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, _, cx| {
        assert_eq!(ids(&collection, cx), ["row-a", "row-b"]);
        assert_eq!(collection.read(cx).events.len(), 2);
        assert_eq!(config.next.get(), 102);
    })
    .unwrap();
}

#[gpui_kit::test]
fn pointer_insert_and_keyboard_duplicate_reorder_remove_emit_once_and_preserve_rows(
    cx: &mut TestAppContext,
) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window
            .within("editor")
            .within("row-a")
            .click("select-row", cx);
        window.within("editor").click("insert-row", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(ids(&collection, cx), ["row-a", "row-100", "row-b"]);
        assert_eq!(collection.read(cx).events.len(), 1);
        assert!(matches!(
            collection.read(cx).events[0].change(),
            RowEditorChange::Insert { ix: 1, .. }
        ));
        assert_eq!(
            collection
                .read(cx)
                .editor
                .read(cx)
                .selected_id()
                .map(AsRef::as_ref),
            Some("row-100")
        );
        window.press("secondary-d", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            ids(&collection, cx),
            ["row-a", "row-100", "row-101", "row-b"]
        );
        assert_eq!(collection.read(cx).events.len(), 2);
        window.press("alt-up", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(
            ids(&collection, cx),
            ["row-a", "row-101", "row-100", "row-b"]
        );
        assert_eq!(collection.read(cx).events.len(), 3);
        assert_eq!(window.within("row-a").find("name").value(), Some("Alpha"));
        assert_eq!(window.within("row-b").find("name").value(), Some("Beta"));
        window.press("delete", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(ids(&collection, cx), ["row-a", "row-100", "row-b"]);
        assert_eq!(collection.read(cx).events.len(), 4);
        assert_eq!(
            collection
                .read(cx)
                .editor
                .read(cx)
                .selected_id()
                .map(AsRef::as_ref),
            Some("row-100")
        );
        // Removal leaves a valid focused row that can accept the next command.
        window.press("alt-down", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        assert_eq!(ids(&collection, cx), ["row-a", "row-b", "row-100"]);
        assert_eq!(collection.read(cx).events.len(), 5);
    });
}

#[gpui_kit::test]
fn controlled_rejection_and_disabled_readonly_behavior_use_the_production_view(
    cx: &mut TestAppContext,
) {
    let (handle, collection) = open(cx);
    cx.update(|cx| collection.update(cx, |this, _| this.accept = false));
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("insert-row", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(ids(&collection, cx), ["row-a", "row-b"]);
        assert_eq!(collection.read(cx).events.len(), 1);
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| state.set_readonly(true, cx));
        window.render_frame(cx);
        window.within("row-a").click("select-row", cx);
        window.press("down", cx);
        window.press("delete", cx);
        window.press("secondary-d", cx);
        window.press("alt-up", cx);
        window.click("insert-row", cx);
        assert_eq!(
            editor.read(cx).selected_id().map(AsRef::as_ref),
            Some("row-b")
        );
        window.within("row-b").click("name", cx);
        window.input("X", cx);
        assert_eq!(window.within("row-b").find("name").value(), Some("Beta"));
        editor.update(cx, |state, cx| state.set_disabled(true, cx));
        window.render_frame(cx);
        window.within("row-a").click("select-row", cx);
        window.press("up", cx);
        window.click("insert-row", cx);
        assert_eq!(
            editor.read(cx).selected_id().map(AsRef::as_ref),
            Some("row-b")
        );
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| assert_eq!(collection.read(cx).events.len(), 1));
}

#[gpui_kit::test]
fn field_edit_roundtrip_and_external_reorder_preserve_field_focus_and_identity(
    cx: &mut TestAppContext,
) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("row-b").click("name", cx);
        window.press("secondary-a", cx);
        window.input("Changed", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        let editor = collection.read(cx).editor.clone();
        assert_eq!(editor.read(cx).rows()[1].name, "Changed");
        let before = collection.read(cx).events.len();
        let mut rows = editor.read(cx).rows().to_vec();
        rows.reverse();
        editor.update(cx, |state, cx| state.set_rows(rows, window, cx).unwrap());
        window.render_frame(cx);
        assert_eq!(window.within("row-b").find("name").focused(), Some(true));
        assert_eq!(window.within("row-b").find("name").value(), Some("Changed"));
        assert_eq!(collection.read(cx).events.len(), before);
        let mut refreshed = editor.read(cx).rows().to_vec();
        refreshed[1].name = "External".into();
        editor.update(cx, |state, cx| {
            state.set_rows(refreshed, window, cx).unwrap()
        });
        window.render_frame(cx);
        assert_eq!(
            window.within("row-a").find("select-row").label(),
            Some("row-a: External")
        );
        assert_eq!(
            window.within("row-a").find("name").value(),
            Some("External")
        );
        assert_eq!(collection.read(cx).events.len(), before);
        // Delete remains a field edit; it must never remove the record.
        window.press("backspace", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        let this = collection.read(cx);
        assert_eq!(this.editor.read(cx).rows().len(), 2);
        assert!(
            this.events
                .iter()
                .all(|event| matches!(event.change(), RowEditorChange::Replace { .. }))
        );
        #[cfg(feature = "component-shape")]
        assert_eq!(
            this.holder.clone().into_original().rows,
            this.editor.read(cx).rows()
        );
    });
}

#[gpui_kit::test]
fn invalid_identity_updates_are_rejected_without_emitting_or_losing_accepted_values(
    cx: &mut TestAppContext,
) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let editor = collection.read(cx).editor.clone();
        let rows = editor.read(cx).rows().to_vec();
        editor.update(cx, |state, cx| {
            assert_eq!(
                state.set_rows(vec![rows[0].clone(), rows[0].clone()], window, cx),
                Err(RowEditorIdentityError::Duplicate { id: "row-a".into() })
            );
            assert_eq!(state.rows(), rows);
            assert_eq!(
                state.set_rows(
                    vec![Record {
                        id: String::new(),
                        name: "Empty ID".into()
                    }],
                    window,
                    cx
                ),
                Err(RowEditorIdentityError::Empty { ix: 0 })
            );
            assert_eq!(state.rows(), rows);
            assert!(matches!(
                state.request_replace(
                    "row-a",
                    Record {
                        id: "changed".into(),
                        name: "Oops".into()
                    },
                    cx
                ),
                Err(RowEditorIdentityError::Changed { .. })
            ));
        });
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| assert!(collection.read(cx).events.is_empty()));
}

#[gpui_kit::test]
fn pointer_menu_commands_request_the_same_typed_operations_as_keyboard(cx: &mut TestAppContext) {
    let (handle, collection) = open(cx);
    for (command_ix, expected, command) in [
        (0usize, vec!["row-a", "row-100", "row-b"], "duplicate"),
        (1, vec!["row-100", "row-a", "row-b"], "move up"),
        (2, vec!["row-a", "row-100", "row-b"], "move down"),
        (4, vec!["row-a", "row-b"], "remove"),
    ] {
        cx.update_window(handle.into(), |_, window, cx| {
            window.render_frame(cx);
            window.click("row-actions", cx);
            window.within("popup-menu").click(command_ix, cx);
        })
        .unwrap();
        cx.run_until_parked();
        cx.update_window(handle.into(), |_, window, cx| {
            window.render_frame(cx);
            assert_eq!(ids(&collection, cx), expected, "{command}");
            assert!(window.try_find("popup-menu").is_none());
        })
        .unwrap();
    }
    cx.update(|cx| {
        let this = collection.read(cx);
        assert_eq!(this.events.len(), 4);
        assert!(matches!(this.events[0].change(), RowEditorChange::Duplicate { source, id, ix: 1 } if source.as_ref() == "row-a" && id.as_ref() == "row-100"));
        assert!(matches!(this.events[1].change(), RowEditorChange::Move { from: 1, to: 0, .. }));
        assert!(matches!(this.events[2].change(), RowEditorChange::Move { from: 0, to: 1, .. }));
        assert!(matches!(this.events[3].change(), RowEditorChange::Remove { ix: 1, .. }));
    });
}

#[gpui_kit::test]
fn removing_the_last_row_preserves_a_keyboard_path_to_insert(cx: &mut TestAppContext) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("row-a").click("select-row", cx);
        window.press("delete", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(ids(&collection, cx), ["row-b"]);
        window.press("delete", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert!(ids(&collection, cx).is_empty());
        assert!(collection.read(cx).editor.read(cx).selected_id().is_none());
        window.press("insert", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        assert_eq!(ids(&collection, cx), ["row-100"]);
        assert_eq!(collection.read(cx).events.len(), 3);
        #[cfg(feature = "component-shape")]
        {
            use gpui_form::runtime::shape::{ValueChange, value_change};
            let this = collection.read(cx);
            let ValueChange::Set(rows) = value_change::<RowEditor<Record, Fields>, Vec<Record>>(
                this.editor.read(cx),
                &this.events[1],
            ) else {
                panic!("empty collection must remain a present typed value");
            };
            assert!(rows.is_empty());
            let holder = OptionalCollectionFormValueHolder { rows: Some(rows) };
            assert_eq!(
                holder.into_original(),
                OptionalCollection {
                    rows: Some(Vec::new())
                }
            );
            let absent = OptionalCollection { rows: None };
            assert_eq!(
                OptionalCollectionFormValueHolder::from(absent.clone()).into_original(),
                absent
            );
        }
    });
}

#[cfg(feature = "component-shape")]
#[test]
fn configured_row_editor_keeps_existing_shape_builder_and_storage_contracts() {
    use gpui_form::runtime::shape::{
        DirectValueStorage, GpuiComponentShapeBuilder, GpuiFormComponentShapePolicy,
    };
    use gpui_form_component::row_editor::RowEditorOptions;
    fn accepts_builder(_: impl GpuiComponentShapeBuilder<RowEditor<Record, Fields>>) {}
    fn accepts_storage<
        Shape: GpuiFormComponentShapePolicy<ValueStoragePolicy = DirectValueStorage>,
    >() {
    }
    accepts_builder(RowEditor::<Record, Fields>::from(
        RowEditorOptions::builder()
            .config(Fields::default())
            .readonly(true)
            .build(),
    ));
    accepts_storage::<RowEditor<Record, Fields>>();
    let form = CollectionForm { rows: Vec::new() };
    assert_eq!(
        CollectionFormFormValueHolder::from(form.clone()).into_original(),
        form
    );
}

#[gpui_kit::test]
fn removing_another_row_preserves_the_surviving_fields_native_focus(cx: &mut TestAppContext) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("row-b").click("name", cx);
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| state.request_remove("row-a", cx).unwrap());
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(ids(&collection, cx), ["row-b"]);
        assert_eq!(window.within("row-b").find("name").focused(), Some(true));
        assert_eq!(window.within("row-b").find("name").value(), Some("Beta"));
        assert_eq!(collection.read(cx).events.len(), 1);
    })
    .unwrap();
}

#[gpui_kit::test]
fn caller_row_ids_share_no_identity_scope_with_toolbar_commands(cx: &mut TestAppContext) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| {
            state
                .set_rows(
                    vec![Record {
                        id: "insert-row".into(),
                        name: "Existing".into(),
                    }],
                    window,
                    cx,
                )
                .unwrap()
        });
        window.render_frame(cx);
        assert_eq!(window.find("insert-row").label(), Some("Insert row"));
        window
            .within("row-list")
            .within("insert-row")
            .click("select-row", cx);
        window.click("insert-row", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        assert_eq!(ids(&collection, cx), ["insert-row", "row-100"]);
        assert_eq!(
            window
                .within("row-list")
                .within("insert-row")
                .find("name")
                .value(),
            Some("Existing")
        );
        assert_eq!(collection.read(cx).events.len(), 1);
    })
    .unwrap();
}

#[cfg(feature = "component-shape")]
#[derive(Clone, Debug, Eq, gpui_form::GpuiForm, PartialEq)]
struct ApplicationCollection {
    #[gpui_form(component(RowEditor::<Record, Fields>, default = Vec::new()))]
    rows: Vec<Record>,
    #[gpui_form(skip)]
    owner: String,
    #[gpui_form(skip)]
    revision: u64,
    #[gpui_form(skip)]
    first: u64,
    #[gpui_form(skip)]
    second: u64,
    #[gpui_form(skip)]
    third: u64,
    #[gpui_form(skip)]
    fourth: u64,
    #[gpui_form(skip)]
    fifth: u64,
}

#[cfg(feature = "component-shape")]
#[gpui_kit::test]
fn edited_reordered_rows_reconstruct_with_application_owned_fields(cx: &mut TestAppContext) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("row-b").click("name", cx);
        window.press("secondary-a", cx);
        window.input("Edited", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("row-b").click("select-row", cx);
        window.press("alt-up", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        let this = collection.read(cx);
        let holder = ApplicationCollectionFormValueHolder { rows: this.holder.rows.clone() };
        let original = holder.into_original("owner".into(), 42, 1, 2, 3, 4, 5).unwrap();
        assert_eq!(original.rows, vec![Record { id: "row-b".into(), name: "Edited".into() }, Record { id: "row-a".into(), name: "Alpha".into() }]);
        assert_eq!((original.owner.as_str(), original.revision, original.first, original.second, original.third, original.fourth, original.fifth), ("owner", 42, 1, 2, 3, 4, 5));
        let (reorder, edits) = this.events.split_last().expect("editing and reordering emit requests");
        assert!(!edits.is_empty());
        assert!(edits.iter().all(|event| matches!(event.change(), RowEditorChange::Replace { id } if id.as_ref() == "row-b")));
        assert_matches!(reorder.change(), RowEditorChange::Move { from: 1, to: 0, .. });
    });
}

#[gpui_kit::test]
fn invalid_factory_identity_recovers_and_unknown_requests_leave_rows_intact(
    cx: &mut TestAppContext,
) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| {
            state
                .set_rows(
                    vec![Record {
                        id: "row-100".into(),
                        name: "Existing".into(),
                    }],
                    window,
                    cx,
                )
                .unwrap();
            assert_eq!(
                state.request_insert(None, cx),
                Err(RowEditorIdentityError::Duplicate {
                    id: "row-100".into()
                })
            );
            assert_eq!(
                state.identity_error(),
                Some(&RowEditorIdentityError::Duplicate {
                    id: "row-100".into()
                })
            );
            assert_eq!(state.rows()[0].name, "Existing");
            state.request_insert(Some("missing"), cx).unwrap();
            state.request_duplicate("missing", cx).unwrap();
            state.request_remove("missing", cx).unwrap();
            state.request_move("missing", 0, cx).unwrap();
            state.request_move("row-100", 9, cx).unwrap();
            state.request_move("row-100", 0, cx).unwrap();
            state
                .request_replace(
                    "missing",
                    Record {
                        id: "missing".into(),
                        name: "Ignored".into(),
                    },
                    cx,
                )
                .unwrap();
        });
        assert!(collection.read(cx).events.is_empty());
        editor.update(cx, |state, cx| state.request_insert(None, cx).unwrap());
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        assert_eq!(ids(&collection, cx), ["row-100", "row-101"]);
        assert_eq!(collection.read(cx).events.len(), 1);
        assert!(
            collection
                .read(cx)
                .editor
                .read(cx)
                .identity_error()
                .is_none()
        );
    });
}

#[gpui_kit::test]
fn external_removal_uses_preceding_row_then_editor_for_keyboard_focus(cx: &mut TestAppContext) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.within("row-b").click("name", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let editor = collection.read(cx).editor.clone();
        let first = editor.read(cx).rows()[0].clone();
        editor.update(cx, |state, cx| {
            state.set_rows(vec![first], window, cx).unwrap()
        });
        window.render_frame(cx);
        assert_eq!(
            editor.read(cx).selected_id().map(AsRef::as_ref),
            Some("row-a")
        );
        editor.update(cx, |state, cx| {
            state.set_rows(Vec::new(), window, cx).unwrap()
        });
        window.render_frame(cx);
        assert!(collection.read(cx).events.is_empty());
        window.press("insert", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        assert_eq!(ids(&collection, cx), ["row-100"]);
        assert_eq!(collection.read(cx).events.len(), 1);
    });
}

#[gpui_kit::test]
fn disabled_and_readonly_gate_every_imperative_edit_and_preserve_rows(cx: &mut TestAppContext) {
    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| {
            let rows = state.rows().to_vec();
            for readonly in [true, false] {
                state.set_readonly(readonly, cx);
                state.set_disabled(!readonly, cx);
                assert!(!state.can_insert());
                assert!(!state.can_duplicate("row-a"));
                state.request_insert(None, cx).unwrap();
                state.request_duplicate("row-a", cx).unwrap();
                state.request_remove("row-a", cx).unwrap();
                state.request_move("row-a", 1, cx).unwrap();
                state
                    .request_replace(
                        "row-a",
                        Record {
                            id: "row-a".into(),
                            name: "Blocked".into(),
                        },
                        cx,
                    )
                    .unwrap();
                assert_eq!(state.rows(), rows);
            }
            state.select_row("row-b", window, cx);
            assert_eq!(state.selected_id().map(AsRef::as_ref), Some("row-a"));
            state.set_disabled(false, cx);
            state.request_duplicate("row-a", cx).unwrap();
        });
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| {
        assert_eq!(ids(&collection, cx), ["row-a", "row-100", "row-b"]);
        assert_eq!(collection.read(cx).events.len(), 1);
    });
}

#[cfg(feature = "component-shape")]
#[gpui_kit::test]
fn invalid_shape_seed_reports_identity_without_replacing_accepted_rows(cx: &mut TestAppContext) {
    use gpui_form::runtime::shape::seed_value_binding_state;

    let (handle, collection) = open(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let editor = collection.read(cx).editor.clone();
        editor.update(cx, |state, cx| {
            let accepted = state.rows().to_vec();
            let invalid = vec![accepted[0].clone(), accepted[0].clone()];
            seed_value_binding_state::<RowEditor<Record, Fields>, Vec<Record>>(
                state,
                Some(&invalid),
                window,
                cx,
            );
            assert_eq!(state.rows(), accepted);
            assert_eq!(
                state.identity_error(),
                Some(&RowEditorIdentityError::Duplicate { id: "row-a".into() })
            );
            seed_value_binding_state::<RowEditor<Record, Fields>, Vec<Record>>(
                state, None, window, cx,
            );
            assert!(state.rows().is_empty());
            assert!(state.identity_error().is_none());
        });
    })
    .unwrap();
    cx.run_until_parked();
    cx.update(|cx| assert!(collection.read(cx).events.is_empty()));
}
