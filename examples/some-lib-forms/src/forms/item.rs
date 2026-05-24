use es_fluent::FluentMessage as _;
use gpui::prelude::FluentBuilder as _;
use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window};
use gpui::{InteractiveElement, ParentElement as _, Styled, Subscription, div};
use gpui_component::ActiveTheme as _;
use gpui_component::Disableable as _;
use gpui_component::form::field;
use gpui_component::form::v_form;
use gpui_component::input::InputEvent;
use gpui_component::separator::Separator;
use gpui_component::v_flex;
use gpui_form_component::shape::{
    ComponentEventOf, ComponentStateOf, FormValueChange, form_value_change,
    seed_value_binding_state,
};
use some_lib::structs::form_action::FormAction;
use some_lib::structs::new_type::*;
const CONTEXT: &str = "ItemForm";
fn localize(cx: &impl std::borrow::Borrow<App>, message: &impl es_fluent::FluentMessage) -> String {
    crate::i18n::localize_message(cx, message)
}
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct ItemForm {
    current_data: ItemFormValueHolder,
    fields: ItemFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for ItemForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for ItemForm {
    fn title(cx: &gpui::App) -> String {
        crate::i18n::localize_label::<Item>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl ItemForm {
    fn on_index_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::input::Input<Age>>>,
        event: &ComponentEventOf<gpui_form_collection::input::Input<Age>, Age>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::input::Input<Age>, Age>(&state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.index = Some(value);
            },
            FormValueChange::Clear => {
                self.current_data.index = None;
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = ItemFormValueHolder::default();
        let index_input = cx.new(|cx| ItemFormComponents::index_input(window, cx));
        let mut _subscriptions =
            vec![cx.subscribe_in(&index_input, window, Self::on_index_input_event)];
        index_input.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<Age>, Age>(
                state,
                current_data.index.as_ref(),
                window,
                cx,
            );
        });
        Self {
            current_data,
            fields: ItemFormFields { index_input },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> Result<Option<Item>, String> {
        match self.current_data.validate() {
            Ok(_) => Ok(ItemFormValueHolder::try_from(self.current_data.clone()).ok()),
            Err(error) => Err(format!("{error:?}")),
        }
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
        on_submit: impl Fn(Result<Option<Item>, String>, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-submit-button", "item-form"))
            .label(label)
            .disabled(self.current_data.validate().is_err())
            .on_click(cx.listener(move |this, _, window, cx| {
                on_submit(this.submit_payload(), window, cx);
            }))
    }
    fn reset_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-reset-button", "item-form"))
            .label(label)
            .on_click(cx.listener(|this, _, window, cx| {
                this.reset_form(window, cx);
            }))
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(Result<Option<Item>, String>, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(cx, localize(cx, &FormAction::Submit), on_submit))
            .child(self.reset_button(cx, localize(cx, &FormAction::Reset)))
    }
}
impl Render for ItemForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.current_data.validate().err();
        v_flex()
            .key_context(CONTEXT)
            .id("item-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label({
                                let message = ItemLabelVariants::Index;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = ItemDescriptionVariants::Index;
                                    localize(cx, &message)
                                };
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.index().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| localize(cx, v))
                                                    .collect::<Vec<_>>()
                                                    .join("\n"),
                                            )
                                        }
                                    })
                                };
                                let error_color = cx.theme().danger;
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                        .when(error.is_some(), |this| {
                                            this.child(
                                                div()
                                                    .text_color(error_color)
                                                    .child(error.clone().unwrap_or_default()),
                                            )
                                        })
                                }
                            })
                            .child(gpui_component::input::Input::new(&self.fields.index_input)),
                    )
                    .child(field().label_indent(false).child(self.action_buttons(
                        cx,
                        |payload, _, _| {
                            let _ = payload;
                        },
                    ))),
            )
            .child(Separator::horizontal())
            .child(format!("value_holder: {:?}", self.current_data))
            .child(format!(
                "into_original: {:?}",
                ItemFormValueHolder::try_from(self.current_data.clone())
            ))
    }
}
