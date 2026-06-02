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
    GpuiComponentEventOf, GpuiComponentStateOf, ValueChange, seed_value_binding_state, value_change,
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
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::input::Input<String>>>,
        event: &GpuiComponentEventOf<gpui_form_collection::input::Input<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::input::Input<String>, String>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.username = value;
            },
            ValueChange::Clear => {
                self.current_data.username = <<gpui_form_collection::input::Input<
                    String,
                > as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    String,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_email_input_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::input::Input<String>>>,
        event: &GpuiComponentEventOf<gpui_form_collection::input::Input<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::input::Input<String>, String>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.email = value;
            },
            ValueChange::Clear => {
                self.current_data.email = ::core::convert::Into::into("test@example.com");
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_age_input_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::input::Input<u32>>>,
        event: &GpuiComponentEventOf<gpui_form_collection::input::Input<u32>, u32>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::input::Input<u32>, u32>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.age = Some(value);
            },
            ValueChange::Clear => {
                self.current_data.age = None;
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_balance_input_event(
        &mut self,
        state: &Entity<
            GpuiComponentStateOf<gpui_form_collection::input::Input<rust_decimal::Decimal>>,
        >,
        event: &GpuiComponentEventOf<
            gpui_form_collection::input::Input<rust_decimal::Decimal>,
            rust_decimal::Decimal,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.balance = value;
            },
            ValueChange::Clear => {
                self.current_data.balance = ::core::convert::Into::into(67);
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_debt_input_event(
        &mut self,
        state: &Entity<
            GpuiComponentStateOf<gpui_form_collection::input::Input<rust_decimal::Decimal>>,
        >,
        event: &GpuiComponentEventOf<
            gpui_form_collection::input::Input<rust_decimal::Decimal>,
            rust_decimal::Decimal,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.debt = value;
            },
            ValueChange::Clear => {
                self.current_data.debt = <<gpui_form_collection::input::Input<
                    rust_decimal::Decimal,
                > as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    rust_decimal::Decimal,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_rating_number_input_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::number_input::NumberInput<u32>>>,
        event: &GpuiComponentEventOf<gpui_form_collection::number_input::NumberInput<u32>, u32>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::number_input::NumberInput<u32>, u32>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.rating = Some(value);
            },
            ValueChange::Clear => {
                self.current_data.rating = None;
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_attention_level_slider_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::slider::Slider>>,
        event: &GpuiComponentEventOf<gpui_form_collection::slider::Slider, f32>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::slider::Slider, f32>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.attention_level = value;
            },
            ValueChange::Clear => {
                self.current_data.attention_level = <<gpui_form_collection::slider::Slider as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    f32,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_brand_color_color_picker_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::color_picker::ColorPicker>>,
        event: &GpuiComponentEventOf<gpui_form_collection::color_picker::ColorPicker, gpui::Hsla>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::color_picker::ColorPicker, gpui::Hsla>(
                state, event,
            )
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.brand_color = Some(value);
            },
            ValueChange::Clear => {
                self.current_data.brand_color = None;
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_otp_code_otp_input_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::otp_input::OtpInput<String>>>,
        event: &GpuiComponentEventOf<gpui_form_collection::otp_input::OtpInput<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::otp_input::OtpInput<String>, String>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.otp_code = value;
            },
            ValueChange::Clear => {
                self.current_data.otp_code = <<gpui_form_collection::otp_input::OtpInput<
                    String,
                > as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    String,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_uploaded_files_file_picker_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_component::file_picker::FilePicker>>,
        event: &GpuiComponentEventOf<
            gpui_form_component::file_picker::FilePicker,
            Vec<std::path::PathBuf>,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_component::file_picker::FilePicker, Vec<std::path::PathBuf>>(
                state, event,
            )
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.uploaded_files = value;
            },
            ValueChange::Clear => {
                self.current_data.uploaded_files = <<gpui_form_component::file_picker::FilePicker as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    Vec<std::path::PathBuf>,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_holiday_range_date_range_picker_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::date_picker::DateRangePicker>>,
        event: &GpuiComponentEventOf<
            gpui_form_collection::date_picker::DateRangePicker,
            (chrono::NaiveDate, chrono::NaiveDate),
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<
                gpui_form_collection::date_picker::DateRangePicker,
                (chrono::NaiveDate, chrono::NaiveDate),
            >(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.holiday_range = Some(value);
            },
            ValueChange::Clear => {
                self.current_data.holiday_range = None;
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_subscribe_newsletter_switch_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::switch::Switch>>,
        event: &GpuiComponentEventOf<gpui_form_collection::switch::Switch, bool>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::switch::Switch, bool>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.subscribe_newsletter = value;
            },
            ValueChange::Clear => {
                self.current_data.subscribe_newsletter = <<gpui_form_collection::switch::Switch as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    bool,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_enable_notifications_checkbox_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::checkbox::Checkbox>>,
        event: &GpuiComponentEventOf<gpui_form_collection::checkbox::Checkbox, bool>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::checkbox::Checkbox, bool>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.enable_notifications = value;
            },
            ValueChange::Clear => {
                self.current_data.enable_notifications = <<gpui_form_collection::checkbox::Checkbox as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    bool,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_preferred_select_event(
        &mut self,
        state: &Entity<
            GpuiComponentStateOf<gpui_form_collection::select::Select<PreferredLanguage>>,
        >,
        event: &GpuiComponentEventOf<
            gpui_form_collection::select::Select<PreferredLanguage>,
            PreferredLanguage,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::select::Select<PreferredLanguage>, PreferredLanguage>(
                state, event,
            )
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.preferred = value;
            },
            ValueChange::Clear => {
                self.current_data.preferred = <<gpui_form_collection::select::Select<
                    PreferredLanguage,
                > as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    PreferredLanguage,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_country_select_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::select::Select<EnumCountry>>>,
        event: &GpuiComponentEventOf<
            gpui_form_collection::select::Select<EnumCountry>,
            EnumCountry,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::select::Select<EnumCountry>, EnumCountry>(
                state, event,
            )
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.country = Some(value);
            },
            ValueChange::Clear => {
                self.current_data.country = None;
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_birth_date_date_picker_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::date_picker::DatePicker>>,
        event: &GpuiComponentEventOf<
            gpui_form_collection::date_picker::DatePicker,
            chrono::NaiveDate,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::date_picker::DatePicker, chrono::NaiveDate>(
                state, event,
            )
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.birth_date = Some(value);
            },
            ValueChange::Clear => {
                self.current_data.birth_date = None;
            },
            ValueChange::Unchanged => {},
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = UserFormValueHolder::default();
        let username = cx.new(|cx| UserFormComponents::username(window, cx));
        let email = cx.new(|cx| UserFormComponents::email(window, cx));
        let age = cx.new(|cx| UserFormComponents::age(window, cx));
        let balance = cx.new(|cx| UserFormComponents::balance(window, cx));
        let debt = cx.new(|cx| UserFormComponents::debt(window, cx));
        let rating = cx.new(|cx| UserFormComponents::rating(window, cx));
        let attention_level = cx.new(|cx| UserFormComponents::attention_level(window, cx));
        let brand_color = cx.new(|cx| UserFormComponents::brand_color(window, cx));
        let otp_code = cx.new(|cx| UserFormComponents::otp_code(window, cx));
        let uploaded_files = cx.new(|cx| UserFormComponents::uploaded_files(window, cx));
        let holiday_range = cx.new(|cx| UserFormComponents::holiday_range(window, cx));
        let subscribe_newsletter =
            cx.new(|cx| UserFormComponents::subscribe_newsletter(window, cx));
        let enable_notifications =
            cx.new(|cx| UserFormComponents::enable_notifications(window, cx));
        let preferred = cx.new(|cx| UserFormComponents::preferred(window, cx));
        let country = cx.new(|cx| UserFormComponents::country(window, cx));
        let birth_date = cx.new(|cx| UserFormComponents::birth_date(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(&username, window, Self::on_username_input_event),
            cx.subscribe_in(&email, window, Self::on_email_input_event),
            cx.subscribe_in(&age, window, Self::on_age_input_event),
            cx.subscribe_in(&balance, window, Self::on_balance_input_event),
            cx.subscribe_in(&debt, window, Self::on_debt_input_event),
            cx.subscribe_in(&rating, window, Self::on_rating_number_input_event),
            cx.subscribe_in(
                &attention_level,
                window,
                Self::on_attention_level_slider_event,
            ),
            cx.subscribe_in(
                &brand_color,
                window,
                Self::on_brand_color_color_picker_event,
            ),
            cx.subscribe_in(&otp_code, window, Self::on_otp_code_otp_input_event),
            cx.subscribe_in(
                &uploaded_files,
                window,
                Self::on_uploaded_files_file_picker_event,
            ),
            cx.subscribe_in(
                &holiday_range,
                window,
                Self::on_holiday_range_date_range_picker_event,
            ),
            cx.subscribe_in(
                &subscribe_newsletter,
                window,
                Self::on_subscribe_newsletter_switch_event,
            ),
            cx.subscribe_in(
                &enable_notifications,
                window,
                Self::on_enable_notifications_checkbox_event,
            ),
            cx.subscribe_in(&preferred, window, Self::on_preferred_select_event),
            cx.subscribe_in(&country, window, Self::on_country_select_event),
            cx.subscribe_in(&birth_date, window, Self::on_birth_date_date_picker_event),
        ];
        username.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<String>, String>(
                state,
                Some(&current_data.username),
                window,
                cx,
            );
        });
        email.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<String>, String>(
                state,
                Some(&current_data.email),
                window,
                cx,
            );
        });
        age.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<u32>, u32>(
                state,
                current_data.age.as_ref(),
                window,
                cx,
            );
        });
        balance.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, Some(&current_data.balance), window, cx);
        });
        debt.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::input::Input<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(state, Some(&current_data.debt), window, cx);
        });
        rating.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::number_input::NumberInput<u32>, u32>(
                state,
                current_data.rating.as_ref(),
                window,
                cx,
            );
        });
        attention_level.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::slider::Slider, f32>(
                state,
                Some(&current_data.attention_level),
                window,
                cx,
            );
        });
        brand_color.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::color_picker::ColorPicker, gpui::Hsla>(
                state,
                current_data.brand_color.as_ref(),
                window,
                cx,
            );
        });
        otp_code.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::otp_input::OtpInput<String>, String>(
                state,
                Some(&current_data.otp_code),
                window,
                cx,
            );
        });
        uploaded_files.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_component::file_picker::FilePicker,
                Vec<std::path::PathBuf>,
            >(state, Some(&current_data.uploaded_files), window, cx);
        });
        holiday_range.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::date_picker::DateRangePicker,
                (chrono::NaiveDate, chrono::NaiveDate),
            >(state, current_data.holiday_range.as_ref(), window, cx);
        });
        subscribe_newsletter.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::switch::Switch, bool>(
                state,
                Some(&current_data.subscribe_newsletter),
                window,
                cx,
            );
        });
        enable_notifications.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::checkbox::Checkbox, bool>(
                state,
                Some(&current_data.enable_notifications),
                window,
                cx,
            );
        });
        preferred.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::select::Select<PreferredLanguage>,
                PreferredLanguage,
            >(state, Some(&current_data.preferred), window, cx);
        });
        country.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::select::Select<EnumCountry>,
                EnumCountry,
            >(state, current_data.country.as_ref(), window, cx);
        });
        birth_date.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_collection::date_picker::DatePicker,
                chrono::NaiveDate,
            >(state, current_data.birth_date.as_ref(), window, cx);
        });
        Self {
            current_data,
            fields: UserFormFields {
                username,
                email,
                age,
                balance,
                debt,
                rating,
                attention_level,
                brand_color,
                otp_code,
                uploaded_files,
                holiday_range,
                subscribe_newsletter,
                enable_notifications,
                preferred,
                country,
                birth_date,
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
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errors = e
                                                .username()
                                                .all()
                                                .map(|v| gpui_es_fluent::localize_message(cx, &v))
                                                .collect::<Vec<_>>();
                                            if errors.is_empty() {
                                                None
                                            } else {
                                                Some(errors.join("\n"))
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
                                        .when(
                                            error.is_some(),
                                            |this| {
                                                this.child(
                                                    div()
                                                        .text_color(error_color)
                                                        .child(error.clone().unwrap_or_default()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(
                                <<gpui_form_collection::input::Input<
                                    String,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::input::Input<
                                        String,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.username),
                            ),
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
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errors = e
                                                .email()
                                                .all()
                                                .map(|v| gpui_es_fluent::localize_message(cx, &v))
                                                .collect::<Vec<_>>();
                                            if errors.is_empty() {
                                                None
                                            } else {
                                                Some(errors.join("\n"))
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
                                        .when(
                                            error.is_some(),
                                            |this| {
                                                this.child(
                                                    div()
                                                        .text_color(error_color)
                                                        .child(error.clone().unwrap_or_default()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(
                                <<gpui_form_collection::input::Input<
                                    String,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::input::Input<
                                        String,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.email),
                            ),
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
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errors = e
                                                .age()
                                                .all()
                                                .map(|v| gpui_es_fluent::localize_message(cx, &v))
                                                .collect::<Vec<_>>();
                                            if errors.is_empty() {
                                                None
                                            } else {
                                                Some(errors.join("\n"))
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
                                        .when(
                                            error.is_some(),
                                            |this| {
                                                this.child(
                                                    div()
                                                        .text_color(error_color)
                                                        .child(error.clone().unwrap_or_default()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(
                                <<gpui_form_collection::input::Input<
                                    u32,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::input::Input<
                                        u32,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.age),
                            ),
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
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errors = e
                                                .balance()
                                                .all()
                                                .map(|v| gpui_es_fluent::localize_message(cx, &v))
                                                .collect::<Vec<_>>();
                                            if errors.is_empty() {
                                                None
                                            } else {
                                                Some(errors.join("\n"))
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
                                        .when(
                                            error.is_some(),
                                            |this| {
                                                this.child(
                                                    div()
                                                        .text_color(error_color)
                                                        .child(error.clone().unwrap_or_default()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(
                                <<gpui_form_collection::input::Input<
                                    rust_decimal::Decimal,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::input::Input<
                                        rust_decimal::Decimal,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.balance),
                            ),
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
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errors = e
                                                .debt()
                                                .all()
                                                .map(|v| gpui_es_fluent::localize_message(cx, &v))
                                                .collect::<Vec<_>>();
                                            if errors.is_empty() {
                                                None
                                            } else {
                                                Some(errors.join("\n"))
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
                                        .when(
                                            error.is_some(),
                                            |this| {
                                                this.child(
                                                    div()
                                                        .text_color(error_color)
                                                        .child(error.clone().unwrap_or_default()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(
                                <<gpui_form_collection::input::Input<
                                    rust_decimal::Decimal,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::input::Input<
                                        rust_decimal::Decimal,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.debt),
                            ),
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
                            .child(
                                <<gpui_form_collection::number_input::NumberInput<
                                    u32,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::number_input::NumberInput<
                                        u32,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.rating),
                            ),
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
                            .child(
                                <<gpui_form_collection::slider::Slider as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::slider::Slider as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.attention_level),
                            ),
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
                            .child(
                                <<gpui_form_collection::color_picker::ColorPicker as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::color_picker::ColorPicker as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.brand_color),
                            ),
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
                            .child(
                                <<gpui_form_collection::otp_input::OtpInput<
                                    String,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::otp_input::OtpInput<
                                        String,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.otp_code),
                            ),
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
                            .child(
                                <<gpui_form_component::file_picker::FilePicker as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_component::file_picker::FilePicker as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.uploaded_files),
                            ),
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
                            .child(
                                <<gpui_form_collection::date_picker::DateRangePicker as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::date_picker::DateRangePicker as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.holiday_range),
                            ),
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
                            .child(
                                <<gpui_form_collection::switch::Switch as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::switch::Switch as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.subscribe_newsletter),
                            ),
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
                            .child(
                                <<gpui_form_collection::checkbox::Checkbox as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::checkbox::Checkbox as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.enable_notifications),
                            ),
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
                            .child(
                                <<gpui_form_collection::select::Select<
                                    PreferredLanguage,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::select::Select<
                                        PreferredLanguage,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.preferred),
                            ),
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
                            .child(
                                <<gpui_form_collection::select::Select<
                                    EnumCountry,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::select::Select<
                                        EnumCountry,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.country),
                            ),
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
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errors = e
                                                .birth_date()
                                                .all()
                                                .map(|v| gpui_es_fluent::localize_message(cx, &v))
                                                .collect::<Vec<_>>();
                                            if errors.is_empty() {
                                                None
                                            } else {
                                                Some(errors.join("\n"))
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
                                        .when(
                                            error.is_some(),
                                            |this| {
                                                this.child(
                                                    div()
                                                        .text_color(error_color)
                                                        .child(error.clone().unwrap_or_default()),
                                                )
                                            },
                                        )
                                }
                            })
                            .child(
                                <<gpui_form_collection::date_picker::DatePicker as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::date_picker::DatePicker as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.birth_date),
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
                    "into_original: incomplete; present_fields: {:?}", self.current_data
                    .present_fields()
                ),
            )
    }
}
