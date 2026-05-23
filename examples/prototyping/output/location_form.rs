use some_lib::structs::location::*;
use es_fluent::FluentMessage as _;
use gpui::{InteractiveElement, ParentElement as _, Styled, Subscription, div};
use gpui::prelude::FluentBuilder as _;
use gpui_component::ActiveTheme as _;
use gpui_component::form::field;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window,
};
use gpui_component::Disableable as _;
use gpui_component::separator::Separator;
use gpui_component::form::v_form;
use gpui_component::v_flex;
use some_lib::structs::form_action::FormAction;
const CONTEXT: &str = "LocationFormForm";
fn localize(
    cx: &impl std::borrow::Borrow<App>,
    message: &impl es_fluent::FluentMessage,
) -> String {
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
    fn on_name_custom_event(
        &mut self,
        state: &Entity<
            <gpui_form_collection::input::InputShape<
                String,
            > as ::gpui_form_component::custom::CustomComponentShape>::State,
        >,
        event: &<gpui_form_collection::input::InputShape<
            String,
        > as ::gpui_form_component::custom::CustomComponentValueAdapter<String>>::Event,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            <gpui_form_collection::input::InputShape<
                String,
            > as ::gpui_form_component::custom::CustomComponentValueAdapter<
                String,
            >>::value_change(&state, event)
        };
        match change {
            ::gpui_form_component::custom::CustomComponentValueChange::Set(value) => {
                self.current_data.name = Some(value);
            }
            ::gpui_form_component::custom::CustomComponentValueChange::Clear => {
                self.current_data.name = None;
            }
            ::gpui_form_component::custom::CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_location_custom_event(
        &mut self,
        state: &Entity<
            <gpui_form_component::infinite_select::InfiniteSelectState<
                Country,
            > as ::gpui_form_component::custom::CustomComponentShape>::State,
        >,
        event: &<gpui_form_component::infinite_select::InfiniteSelectState<
            Country,
        > as ::gpui_form_component::custom::CustomComponentValueAdapter<Country>>::Event,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            <gpui_form_component::infinite_select::InfiniteSelectState<
                Country,
            > as ::gpui_form_component::custom::CustomComponentValueAdapter<
                Country,
            >>::value_change(&state, event)
        };
        match change {
            ::gpui_form_component::custom::CustomComponentValueChange::Set(value) => {
                self.current_data.location = value;
            }
            ::gpui_form_component::custom::CustomComponentValueChange::Clear => {}
            ::gpui_form_component::custom::CustomComponentValueChange::Unchanged => {}
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = LocationFormFormValueHolder::default();
        let name_custom = cx
            .new(|cx| LocationFormFormComponents::name_custom(window, cx));
        let location_custom = cx
            .new(|cx| LocationFormFormComponents::location_custom(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(& name_custom, window, Self::on_name_custom_event), cx
            .subscribe_in(& location_custom, window, Self::on_location_custom_event)
        ];
        name_custom
            .update(
                cx,
                |state, cx| {
                    <gpui_form_collection::input::InputShape<
                        String,
                    > as ::gpui_form_component::custom::CustomComponentValueAdapter<
                        String,
                    >>::set_state_value(state, current_data.name.as_ref(), window, cx);
                },
            );
        location_custom
            .update(
                cx,
                |state, cx| {
                    <gpui_form_component::infinite_select::InfiniteSelectState<
                        Country,
                    > as ::gpui_form_component::custom::CustomComponentValueAdapter<
                        Country,
                    >>::set_state_value(state, Some(&current_data.location), window, cx);
                },
            );
        Self {
            current_data,
            fields: LocationFormFormFields {
                name_custom,
                location_custom,
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
        gpui_component::button::Button::new(
                format!("{}-submit-button", "location_form-form"),
            )
            .label(label)
            .disabled(false)
            .on_click(
                cx
                    .listener(move |this, _, window, cx| {
                        on_submit(this.submit_payload(), window, cx);
                    }),
            )
    }
    fn reset_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(
                format!("{}-reset-button", "location_form-form"),
            )
            .label(label)
            .on_click(
                cx
                    .listener(|this, _, window, cx| {
                        this.reset_form(window, cx);
                    }),
            )
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
                            .child(
                                gpui_component::input::Input::new(&self.fields.name_custom),
                            ),
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
                                    &self.fields.location_custom,
                                ),
                            ),
                    )
                    .child(
                        field()
                            .label_indent(false)
                            .child(
                                self
                                    .action_buttons(
                                        cx,
                                        |payload, _, _| {
                                            let _ = payload;
                                        },
                                    ),
                            ),
                    ),
            )
            .child(Separator::horizontal())
            .child(format!("value_holder: {:?}", self.current_data))
            .child(
                format!(
                    "into_original: {:?}", LocationFormFormValueHolder::try_from(self
                    .current_data.clone())
                ),
            )
    }
}
