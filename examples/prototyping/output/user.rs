use gpui::prelude::FluentBuilder as _;
use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window};
use gpui::{InteractiveElement, ParentElement as _, Styled, Subscription, div};
use gpui_component::ActiveTheme as _;
use gpui_component::Disableable as _;
use gpui_component::form::field;
use gpui_component::form::v_form;
use gpui_component::separator::Separator;
use gpui_component::v_flex;
use gpui_form::runtime::shape::{
    ComponentEventOf, ComponentStateOf, FormValueChange, form_value_change,
    seed_value_binding_state,
};
use some_lib::structs::form_action::FormAction;
use some_lib::structs::user::*;
const CONTEXT: &str = "UserForm";
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
pub struct UserForm {
    current_data: UserFormValueHolder,
    fields: UserFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for UserForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for UserForm {
    fn title(cx: &gpui::App) -> String {
        gpui_es_fluent::localize_label::<User>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl UserForm {
    fn on_username_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::input::Input<String>>>,
        event: &ComponentEventOf<gpui_form_collection::input::Input<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::input::Input<String>, String>(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.username = value;
            },
            FormValueChange::Clear => {
                self.current_data.username = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_email_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::input::Input<String>>>,
        event: &ComponentEventOf<gpui_form_collection::input::Input<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::input::Input<String>, String>(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.email = value;
            },
            FormValueChange::Clear => {
                self.current_data.email = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_age_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::input::Input<u32>>>,
        event: &ComponentEventOf<gpui_form_collection::input::Input<u32>, u32>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::input::Input<u32>, u32>(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.age = Some(value);
            },
            FormValueChange::Clear => {
                self.current_data.age = None;
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_balance_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::input::Input<rust_decimal::Decimal>>>,
        event: &ComponentEventOf<
            gpui_form_collection::input::Input<rust_decimal::Decimal>,
            rust_decimal::Decimal,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.balance = value;
            },
            FormValueChange::Clear => {
                self.current_data.balance = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_debt_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::input::Input<rust_decimal::Decimal>>>,
        event: &ComponentEventOf<
            gpui_form_collection::input::Input<rust_decimal::Decimal>,
            rust_decimal::Decimal,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.debt = value;
            },
            FormValueChange::Clear => {
                self.current_data.debt = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_rating_number_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::number_input::NumberInput<u32>>>,
        event: &ComponentEventOf<gpui_form_collection::number_input::NumberInput<u32>, u32>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::number_input::NumberInput<u32>, u32>(
                state, event,
            )
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.rating = Some(value);
            },
            FormValueChange::Clear => {
                self.current_data.rating = None;
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_attention_level_slider_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::slider::Slider>>,
        event: &ComponentEventOf<gpui_form_collection::slider::Slider, f32>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::slider::Slider, f32>(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.attention_level = value;
            },
            FormValueChange::Clear => {
                self.current_data.attention_level = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_brand_color_color_picker_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::color_picker::ColorPicker>>,
        event: &ComponentEventOf<gpui_form_collection::color_picker::ColorPicker, gpui::Hsla>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::color_picker::ColorPicker, gpui::Hsla>(
                state, event,
            )
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.brand_color = Some(value);
            },
            FormValueChange::Clear => {
                self.current_data.brand_color = None;
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_otp_code_otp_input_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::otp_input::OtpInput<String>>>,
        event: &ComponentEventOf<gpui_form_collection::otp_input::OtpInput<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::otp_input::OtpInput<String>, String>(
                state, event,
            )
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.otp_code = value;
            },
            FormValueChange::Clear => {
                self.current_data.otp_code = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_uploaded_files_file_picker_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_component::file_picker::FilePicker>>,
        event: &ComponentEventOf<
            gpui_form_component::file_picker::FilePicker,
            Vec<std::path::PathBuf>,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_component::file_picker::FilePicker, Vec<std::path::PathBuf>>(
                state, event,
            )
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.uploaded_files = value;
            },
            FormValueChange::Clear => {
                self.current_data.uploaded_files = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_holiday_range_date_range_picker_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::date_picker::DateRangePicker>>,
        event: &ComponentEventOf<
            gpui_form_collection::date_picker::DateRangePicker,
            (chrono::NaiveDate, chrono::NaiveDate),
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<
                gpui_form_collection::date_picker::DateRangePicker,
                (chrono::NaiveDate, chrono::NaiveDate),
            >(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.holiday_range = Some(value);
            },
            FormValueChange::Clear => {
                self.current_data.holiday_range = None;
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_subscribe_newsletter_switch_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::switch::Switch>>,
        event: &ComponentEventOf<gpui_form_collection::switch::Switch, bool>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::switch::Switch, bool>(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.subscribe_newsletter = value;
            },
            FormValueChange::Clear => {
                self.current_data.subscribe_newsletter = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_enable_notifications_checkbox_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::checkbox::Checkbox>>,
        event: &ComponentEventOf<gpui_form_collection::checkbox::Checkbox, bool>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::checkbox::Checkbox, bool>(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.enable_notifications = value;
            },
            FormValueChange::Clear => {
                self.current_data.enable_notifications = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_preferred_select_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::select::Select<PreferredLanguage>>>,
        event: &ComponentEventOf<
            gpui_form_collection::select::Select<PreferredLanguage>,
            PreferredLanguage,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<
                gpui_form_collection::select::Select<PreferredLanguage>,
                PreferredLanguage,
            >(state, event)
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.preferred = value;
            },
            FormValueChange::Clear => {
                self.current_data.preferred = ::core::default::Default::default();
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_country_select_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::select::Select<EnumCountry>>>,
        event: &ComponentEventOf<gpui_form_collection::select::Select<EnumCountry>, EnumCountry>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::select::Select<EnumCountry>, EnumCountry>(
                state, event,
            )
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.country = Some(value);
            },
            FormValueChange::Clear => {
                self.current_data.country = None;
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn on_birth_date_date_picker_event(
        &mut self,
        state: &Entity<ComponentStateOf<gpui_form_collection::date_picker::DatePicker>>,
        event: &ComponentEventOf<gpui_form_collection::date_picker::DatePicker, chrono::NaiveDate>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            form_value_change::<gpui_form_collection::date_picker::DatePicker, chrono::NaiveDate>(
                state, event,
            )
        };
        match form_change {
            FormValueChange::Set(value) => {
                self.current_data.birth_date = Some(value);
            },
            FormValueChange::Clear => {
                self.current_data.birth_date = None;
            },
            FormValueChange::Unchanged => {},
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = UserFormValueHolder::default();
        let username_input = cx.new(|cx| UserFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| UserFormComponents::email_input(window, cx));
        let age_input = cx.new(|cx| UserFormComponents::age_input(window, cx));
        let balance_input = cx.new(|cx| UserFormComponents::balance_input(window, cx));
        let debt_input = cx.new(|cx| UserFormComponents::debt_input(window, cx));
        let rating_number_input = cx.new(|cx| UserFormComponents::rating_number_input(window, cx));
        let attention_level_slider =
            cx.new(|cx| UserFormComponents::attention_level_slider(window, cx));
        let brand_color_color_picker =
            cx.new(|cx| UserFormComponents::brand_color_color_picker(window, cx));
        let otp_code_otp_input = cx.new(|cx| UserFormComponents::otp_code_otp_input(window, cx));
        let uploaded_files_file_picker =
            cx.new(|cx| UserFormComponents::uploaded_files_file_picker(window, cx));
        let holiday_range_date_range_picker =
            cx.new(|cx| UserFormComponents::holiday_range_date_range_picker(window, cx));
        let subscribe_newsletter_switch =
            cx.new(|cx| UserFormComponents::subscribe_newsletter_switch(window, cx));
        let enable_notifications_checkbox =
            cx.new(|cx| UserFormComponents::enable_notifications_checkbox(window, cx));
        let preferred_select = cx.new(|cx| UserFormComponents::preferred_select(window, cx));
        let country_select = cx.new(|cx| UserFormComponents::country_select(window, cx));
        let birth_date_date_picker =
            cx.new(|cx| UserFormComponents::birth_date_date_picker(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(&username_input, window, Self::on_username_input_event),
            cx.subscribe_in(&email_input, window, Self::on_email_input_event),
            cx.subscribe_in(&age_input, window, Self::on_age_input_event),
            cx.subscribe_in(&balance_input, window, Self::on_balance_input_event),
            cx.subscribe_in(&debt_input, window, Self::on_debt_input_event),
            cx.subscribe_in(
                &rating_number_input,
                window,
                Self::on_rating_number_input_event,
            ),
            cx.subscribe_in(
                &attention_level_slider,
                window,
                Self::on_attention_level_slider_event,
            ),
            cx.subscribe_in(
                &brand_color_color_picker,
                window,
                Self::on_brand_color_color_picker_event,
            ),
            cx.subscribe_in(
                &otp_code_otp_input,
                window,
                Self::on_otp_code_otp_input_event,
            ),
            cx.subscribe_in(
                &uploaded_files_file_picker,
                window,
                Self::on_uploaded_files_file_picker_event,
            ),
            cx.subscribe_in(
                &holiday_range_date_range_picker,
                window,
                Self::on_holiday_range_date_range_picker_event,
            ),
            cx.subscribe_in(
                &subscribe_newsletter_switch,
                window,
                Self::on_subscribe_newsletter_switch_event,
            ),
            cx.subscribe_in(
                &enable_notifications_checkbox,
                window,
                Self::on_enable_notifications_checkbox_event,
            ),
            cx.subscribe_in(&preferred_select, window, Self::on_preferred_select_event),
            cx.subscribe_in(&country_select, window, Self::on_country_select_event),
            cx.subscribe_in(
                &birth_date_date_picker,
                window,
                Self::on_birth_date_date_picker_event,
            ),
        ];
        username_input.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<String>, String>(
                state,
                Some(&current_data.username),
                window,
                cx,
            );
        });
        email_input.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<String>, String>(
                state,
                Some(&current_data.email),
                window,
                cx,
            );
        });
        age_input.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<u32>, u32>(
                state,
                current_data.age.as_ref(),
                window,
                cx,
            );
        });
        balance_input.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, Some(&current_data.balance), window, cx);
        });
        debt_input.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, Some(&current_data.debt), window, cx);
        });
        rating_number_input.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::number_input::NumberInput<u32>, u32>(
                state,
                current_data.rating.as_ref(),
                window,
                cx,
            );
        });
        attention_level_slider.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::slider::Slider, f32>(
                state,
                Some(&current_data.attention_level),
                window,
                cx,
            );
        });
        brand_color_color_picker.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::color_picker::ColorPicker, gpui::Hsla>(
                state,
                current_data.brand_color.as_ref(),
                window,
                cx,
            );
        });
        otp_code_otp_input.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::otp_input::OtpInput<String>, String>(
                state,
                Some(&current_data.otp_code),
                window,
                cx,
            );
        });
        uploaded_files_file_picker.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_component::file_picker::FilePicker,
                Vec<std::path::PathBuf>,
            >(state, Some(&current_data.uploaded_files), window, cx);
        });
        holiday_range_date_range_picker.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::date_picker::DateRangePicker,
                (chrono::NaiveDate, chrono::NaiveDate),
            >(state, current_data.holiday_range.as_ref(), window, cx);
        });
        subscribe_newsletter_switch.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::switch::Switch, bool>(
                state,
                Some(&current_data.subscribe_newsletter),
                window,
                cx,
            );
        });
        enable_notifications_checkbox.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::checkbox::Checkbox, bool>(
                state,
                Some(&current_data.enable_notifications),
                window,
                cx,
            );
        });
        preferred_select.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::select::Select<PreferredLanguage>,
                PreferredLanguage,
            >(state, Some(&current_data.preferred), window, cx);
        });
        country_select.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::select::Select<EnumCountry>,
                EnumCountry,
            >(state, current_data.country.as_ref(), window, cx);
        });
        birth_date_date_picker.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::date_picker::DatePicker,
                chrono::NaiveDate,
            >(state, current_data.birth_date.as_ref(), window, cx);
        });
        Self {
            current_data,
            fields: UserFormFields {
                username_input,
                email_input,
                age_input,
                balance_input,
                debt_input,
                rating_number_input,
                attention_level_slider,
                brand_color_color_picker,
                otp_code_otp_input,
                uploaded_files_file_picker,
                holiday_range_date_range_picker,
                subscribe_newsletter_switch,
                enable_notifications_checkbox,
                preferred_select,
                country_select,
                birth_date_date_picker,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> Result<UserFormValueHolder, String> {
        match self.current_data.validate() {
            Ok(_) => Ok(self.current_data.clone()),
            Err(error) => Err(format!("{error:?}")),
        }
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui::SharedString>,
        on_submit: impl Fn(Result<UserFormValueHolder, String>, &mut Window, &mut Context<Self>)
        + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-submit-button", "user-form"))
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
        gpui_component::button::Button::new(format!("{}-reset-button", "user-form"))
            .label(label)
            .on_click(cx.listener(|this, _, window, cx| {
                this.reset_form(window, cx);
            }))
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(Result<UserFormValueHolder, String>, &mut Window, &mut Context<Self>)
        + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(
                cx,
                gpui_es_fluent::localize_message(cx, &FormAction::Submit),
                on_submit,
            ))
            .child(self.reset_button(cx, gpui_es_fluent::localize_message(cx, &FormAction::Reset)))
    }
}
impl Render for UserForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation_errors = self.current_data.validate().err();
        v_flex()
            .key_context(CONTEXT)
            .id("user-form")
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
                                let message = UserLabelVariants::Username;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Username;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.username().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| {
                                                        gpui_es_fluent::localize_message(cx, v)
                                                    })
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
                            .child(<gpui_component::input::Input>::new(
                                &self.fields.username_input,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Email;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Email;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.email().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| {
                                                        gpui_es_fluent::localize_message(cx, v)
                                                    })
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
                            .child(<gpui_component::input::Input>::new(
                                &self.fields.email_input,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Age;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Age;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.age().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| {
                                                        gpui_es_fluent::localize_message(cx, v)
                                                    })
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
                            .child(<gpui_component::input::Input>::new(&self.fields.age_input)),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Balance;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Balance;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.balance().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| {
                                                        gpui_es_fluent::localize_message(cx, v)
                                                    })
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
                            .child(<gpui_component::input::Input>::new(
                                &self.fields.balance_input,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Debt;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Debt;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                let error = {
                                    validation_errors.as_ref().and_then(|e| {
                                        let errs = e.debt().all();
                                        if errs.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                errs.iter()
                                                    .map(|v| {
                                                        gpui_es_fluent::localize_message(cx, v)
                                                    })
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
                            .child(<gpui_component::input::Input>::new(&self.fields.debt_input)),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Rating;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Rating;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_form_collection::number_input::NumberInputField>::new(
                                &self.fields.rating_number_input,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::AttentionLevel;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::AttentionLevel;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_component::slider::Slider>::new(
                                &self.fields.attention_level_slider,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::BrandColor;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::BrandColor;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_component::color_picker::ColorPicker>::new(
                                &self.fields.brand_color_color_picker,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::OtpCode;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::OtpCode;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_form_collection::otp_input::OtpInputField>::new(
                                &self.fields.otp_code_otp_input,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::UploadedFiles;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::UploadedFiles;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_form_component::file_picker::FilePicker>::new(
                                &self.fields.uploaded_files_file_picker,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::HolidayRange;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::HolidayRange;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_component::date_picker::DatePicker>::new(
                                &self.fields.holiday_range_date_range_picker,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::SubscribeNewsletter;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::SubscribeNewsletter;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_form_collection::switch::SwitchField>::new(
                                &self.fields.subscribe_newsletter_switch,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::EnableNotifications;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::EnableNotifications;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_form_collection::checkbox::CheckboxField>::new(
                                &self.fields.enable_notifications_checkbox,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Preferred;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Preferred;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_component::select::Select<_>>::new(
                                &self.fields.preferred_select,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Country;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Country;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_component::select::Select<_>>::new(
                                &self.fields.country_select,
                            )),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::BirthDate;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::BirthDate;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(<gpui_component::date_picker::DatePicker>::new(
                                &self.fields.birth_date_date_picker,
                            )),
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
                "into_original: incomplete; present_fields: {:?}",
                self.current_data.present_fields()
            ))
    }
}
