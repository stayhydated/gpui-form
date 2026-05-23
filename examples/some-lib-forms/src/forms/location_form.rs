use es_fluent::FluentMessage as _;
use gpui::prelude::FluentBuilder as _;
use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window};
use gpui::{InteractiveElement, ParentElement as _, Styled, Subscription, div};
use gpui_component::ActiveTheme as _;
use gpui_component::Disableable as _;
use gpui_component::form::field;
use gpui_component::form::v_form;
use gpui_component::separator::Separator;
use gpui_component::v_flex;
use gpui_form_component::custom::{
    CustomComponentEventOf, CustomComponentStateOf, CustomComponentValueChange,
    custom_value_change, set_custom_state_value,
};
use some_lib::structs::form_action::FormAction;
use some_lib::structs::location::*;
const CONTEXT: &str = "LocationFormForm";
fn localize(cx: &impl std::borrow::Borrow<App>, message: &impl es_fluent::FluentMessage) -> String {
    crate::i18n::localize_message(cx, message)
}
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct LocationFormForm {
    current_data: LocationFormFormValueHolder,
    fields: LocationFormFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for LocationFormForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for LocationFormForm {
    fn title(cx: &gpui::App) -> String {
        crate::i18n::localize_label::<LocationForm>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl LocationFormForm {
    fn on_name_input_event(
        &mut self,
        state: &Entity<CustomComponentStateOf<gpui_form_collection::input::InputShape<String>>>,
        event: &CustomComponentEventOf<gpui_form_collection::input::InputShape<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<gpui_form_collection::input::InputShape<String>, String>(
                &state, event,
            )
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.name = Some(value);
            },
            CustomComponentValueChange::Clear => {
                self.current_data.name = None;
            },
            CustomComponentValueChange::Unchanged => {},
        }
    }
    fn on_location_infinite_select_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<
                gpui_form_component::infinite_select::InfiniteSelectState<Country>,
            >,
        >,
        event: &CustomComponentEventOf<
            gpui_form_component::infinite_select::InfiniteSelectState<Country>,
            Country,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_component::infinite_select::InfiniteSelectState<Country>,
                Country,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.location = value;
            },
            CustomComponentValueChange::Clear => {},
            CustomComponentValueChange::Unchanged => {},
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = LocationFormFormValueHolder::default();
        let name_input = cx.new(|cx| LocationFormFormComponents::name_input(window, cx));
        let location_infinite_select =
            cx.new(|cx| LocationFormFormComponents::location_infinite_select(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(&name_input, window, Self::on_name_input_event),
            cx.subscribe_in(
                &location_infinite_select,
                window,
                Self::on_location_infinite_select_event,
            ),
        ];
        name_input.update(cx, |state, cx| {
            set_custom_state_value::<gpui_form_collection::input::InputShape<String>, String>(
                state,
                current_data.name.as_ref(),
                window,
                cx,
            );
        });
        location_infinite_select.update(cx, |state, cx| {
            set_custom_state_value::<
                gpui_form_component::infinite_select::InfiniteSelectState<Country>,
                Country,
            >(state, Some(&current_data.location), window, cx);
        });
        Self {
            current_data,
            fields: LocationFormFormFields {
                name_input,
                location_infinite_select,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> LocationForm {
        self.current_data.clone().into()
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
        on_submit: impl Fn(LocationForm, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-submit-button", "location_form-form"))
            .label(label)
            .disabled(false)
            .on_click(cx.listener(move |this, _, window, cx| {
                on_submit(this.submit_payload(), window, cx);
            }))
    }
    fn reset_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-reset-button", "location_form-form"))
            .label(label)
            .on_click(cx.listener(|this, _, window, cx| {
                this.reset_form(window, cx);
            }))
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(LocationForm, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(cx, localize(cx, &FormAction::Submit), on_submit))
            .child(self.reset_button(cx, localize(cx, &FormAction::Reset)))
    }
}
impl Render for LocationFormForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("location_form-form")
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
                                let message = LocationFormLabelVariants::Name;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = LocationFormDescriptionVariants::Name;
                                    localize(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(gpui_component::input::Input::new(&self.fields.name_input)),
                    )
                    .child(
                        field()
                            .label({
                                let message = LocationFormLabelVariants::Location;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = LocationFormDescriptionVariants::Location;
                                    localize(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(
                                gpui_form_component::infinite_select::InfiniteSelectField::new(
                                    &self.fields.location_infinite_select,
                                ),
                            ),
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
                LocationFormFormValueHolder::try_from(self.current_data.clone())
            ))
    }
}
