use some_lib::structs::user::*;
use es_fluent::FluentMessage as _;
use gpui::{InteractiveElement, ParentElement as _, Styled, Subscription, div};
use gpui::prelude::FluentBuilder as _;
use gpui_component::ActiveTheme as _;
use gpui_component::form::field;
use gpui_form_component::custom::{
    CustomComponentEventOf, CustomComponentStateOf, CustomComponentValueChange,
    custom_value_change, set_custom_state_value,
};
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window,
};
use gpui_component::Disableable as _;
use gpui_component::separator::Separator;
use gpui_component::form::v_form;
use gpui_component::v_flex;
use some_lib::structs::form_action::FormAction;
const CONTEXT: &str = "UserForm";
fn localize(
    cx: &impl std::borrow::Borrow<App>,
    message: &impl es_fluent::FluentMessage,
) -> String {
    crate::i18n::localize_message(cx, message)
}
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
        crate::i18n::localize_label::<User>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl UserForm {
    fn on_username_input_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<gpui_form_collection::input::InputShape<String>>,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::input::InputShape<String>,
            String,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::input::InputShape<String>,
                String,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.username = Some(value);
            }
            CustomComponentValueChange::Clear => {
                self.current_data.username = None;
            }
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_email_input_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<gpui_form_collection::input::InputShape<String>>,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::input::InputShape<String>,
            String,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::input::InputShape<String>,
                String,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.email = Some(value);
            }
            CustomComponentValueChange::Clear => {
                self.current_data.email = None;
            }
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_age_input_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<gpui_form_collection::input::InputShape<u32>>,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::input::InputShape<u32>,
            u32,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::input::InputShape<u32>,
                u32,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.age = Some(value);
            }
            CustomComponentValueChange::Clear => {
                self.current_data.age = None;
            }
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_balance_input_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<
                gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
            >,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
            rust_decimal::Decimal,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.balance = Some(value);
            }
            CustomComponentValueChange::Clear => {
                self.current_data.balance = None;
            }
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_debt_input_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<
                gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
            >,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
            rust_decimal::Decimal,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
                rust_decimal::Decimal,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.debt = Some(value);
            }
            CustomComponentValueChange::Clear => {
                self.current_data.debt = None;
            }
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_subscribe_newsletter_switch_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<gpui_form_collection::switch::SwitchShape>,
        >,
        event: &CustomComponentEventOf<gpui_form_collection::switch::SwitchShape, bool>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::switch::SwitchShape,
                bool,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.subscribe_newsletter = value;
            }
            CustomComponentValueChange::Clear => {}
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_enable_notifications_checkbox_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<gpui_form_collection::checkbox::CheckboxShape>,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::checkbox::CheckboxShape,
            bool,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::checkbox::CheckboxShape,
                bool,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.enable_notifications = value;
            }
            CustomComponentValueChange::Clear => {}
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_preferred_select_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<
                gpui_form_collection::select::SelectShape<PreferredLanguage>,
            >,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::select::SelectShape<PreferredLanguage>,
            PreferredLanguage,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::select::SelectShape<PreferredLanguage>,
                PreferredLanguage,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.preferred = value;
            }
            CustomComponentValueChange::Clear => {}
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_country_select_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<
                gpui_form_collection::select::SelectShape<
                    EnumCountry,
                    ::gpui_component::select::SearchableVec<EnumCountry>,
                >,
            >,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::select::SelectShape<
                EnumCountry,
                ::gpui_component::select::SearchableVec<EnumCountry>,
            >,
            EnumCountry,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::select::SelectShape<
                    EnumCountry,
                    ::gpui_component::select::SearchableVec<EnumCountry>,
                >,
                EnumCountry,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.country = Some(value);
            }
            CustomComponentValueChange::Clear => {
                self.current_data.country = None;
            }
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn on_birth_date_input_event(
        &mut self,
        state: &Entity<
            CustomComponentStateOf<
                gpui_form_collection::input::InputShape<chrono::NaiveDate>,
            >,
        >,
        event: &CustomComponentEventOf<
            gpui_form_collection::input::InputShape<chrono::NaiveDate>,
            chrono::NaiveDate,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let change = {
            let state = state.read(_cx);
            custom_value_change::<
                gpui_form_collection::input::InputShape<chrono::NaiveDate>,
                chrono::NaiveDate,
            >(&state, event)
        };
        match change {
            CustomComponentValueChange::Set(value) => {
                self.current_data.birth_date = Some(value);
            }
            CustomComponentValueChange::Clear => {
                self.current_data.birth_date = None;
            }
            CustomComponentValueChange::Unchanged => {}
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = UserFormValueHolder::default();
        let username_input = cx.new(|cx| UserFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| UserFormComponents::email_input(window, cx));
        let age_input = cx.new(|cx| UserFormComponents::age_input(window, cx));
        let balance_input = cx.new(|cx| UserFormComponents::balance_input(window, cx));
        let debt_input = cx.new(|cx| UserFormComponents::debt_input(window, cx));
        let subscribe_newsletter_switch = cx
            .new(|cx| UserFormComponents::subscribe_newsletter_switch(window, cx));
        let enable_notifications_checkbox = cx
            .new(|cx| UserFormComponents::enable_notifications_checkbox(window, cx));
        let preferred_select = cx
            .new(|cx| UserFormComponents::preferred_select(window, cx));
        let country_select = cx.new(|cx| UserFormComponents::country_select(window, cx));
        let birth_date_input = cx
            .new(|cx| UserFormComponents::birth_date_input(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(& username_input, window, Self::on_username_input_event), cx
            .subscribe_in(& email_input, window, Self::on_email_input_event), cx
            .subscribe_in(& age_input, window, Self::on_age_input_event), cx
            .subscribe_in(& balance_input, window, Self::on_balance_input_event), cx
            .subscribe_in(& debt_input, window, Self::on_debt_input_event), cx
            .subscribe_in(& subscribe_newsletter_switch, window,
            Self::on_subscribe_newsletter_switch_event), cx.subscribe_in(&
            enable_notifications_checkbox, window,
            Self::on_enable_notifications_checkbox_event), cx.subscribe_in(&
            preferred_select, window, Self::on_preferred_select_event), cx.subscribe_in(&
            country_select, window, Self::on_country_select_event), cx.subscribe_in(&
            birth_date_input, window, Self::on_birth_date_input_event)
        ];
        username_input
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::input::InputShape<String>,
                        String,
                    >(state, current_data.username.as_ref(), window, cx);
                },
            );
        email_input
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::input::InputShape<String>,
                        String,
                    >(state, current_data.email.as_ref(), window, cx);
                },
            );
        age_input
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::input::InputShape<u32>,
                        u32,
                    >(state, current_data.age.as_ref(), window, cx);
                },
            );
        balance_input
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
                        rust_decimal::Decimal,
                    >(state, current_data.balance.as_ref(), window, cx);
                },
            );
        debt_input
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::input::InputShape<rust_decimal::Decimal>,
                        rust_decimal::Decimal,
                    >(state, current_data.debt.as_ref(), window, cx);
                },
            );
        subscribe_newsletter_switch
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::switch::SwitchShape,
                        bool,
                    >(state, Some(&current_data.subscribe_newsletter), window, cx);
                },
            );
        enable_notifications_checkbox
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::checkbox::CheckboxShape,
                        bool,
                    >(state, Some(&current_data.enable_notifications), window, cx);
                },
            );
        preferred_select
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::select::SelectShape<PreferredLanguage>,
                        PreferredLanguage,
                    >(state, Some(&current_data.preferred), window, cx);
                },
            );
        country_select
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::select::SelectShape<
                            EnumCountry,
                            ::gpui_component::select::SearchableVec<EnumCountry>,
                        >,
                        EnumCountry,
                    >(state, current_data.country.as_ref(), window, cx);
                },
            );
        birth_date_input
            .update(
                cx,
                |state, cx| {
                    set_custom_state_value::<
                        gpui_form_collection::input::InputShape<chrono::NaiveDate>,
                        chrono::NaiveDate,
                    >(state, current_data.birth_date.as_ref(), window, cx);
                },
            );
        Self {
            current_data,
            fields: UserFormFields {
                username_input,
                email_input,
                age_input,
                balance_input,
                debt_input,
                subscribe_newsletter_switch,
                enable_notifications_checkbox,
                preferred_select,
                country_select,
                birth_date_input,
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
        on_submit: impl Fn(
            Result<UserFormValueHolder, String>,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> gpui_component::button::Button {
        gpui_component::button::Button::new(format!("{}-submit-button", "user-form"))
            .label(label)
            .disabled(self.current_data.validate().is_err())
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
        gpui_component::button::Button::new(format!("{}-reset-button", "user-form"))
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
        on_submit: impl Fn(
            Result<UserFormValueHolder, String>,
            &mut Window,
            &mut Context<Self>,
        ) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(cx, localize(cx, &FormAction::Submit), on_submit))
            .child(self.reset_button(cx, localize(cx, &FormAction::Reset)))
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
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Username;
                                    localize(cx, &message)
                                };
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errs = e.username().all();
                                            if errs.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    errs
                                                        .iter()
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
                                gpui_component::input::Input::new(
                                    &self.fields.username_input,
                                ),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Email;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Email;
                                    localize(cx, &message)
                                };
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errs = e.email().all();
                                            if errs.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    errs
                                                        .iter()
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
                                gpui_component::input::Input::new(&self.fields.email_input),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Age;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Age;
                                    localize(cx, &message)
                                };
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errs = e.age().all();
                                            if errs.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    errs
                                                        .iter()
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
                                gpui_component::input::Input::new(&self.fields.age_input),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Balance;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Balance;
                                    localize(cx, &message)
                                };
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errs = e.balance().all();
                                            if errs.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    errs
                                                        .iter()
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
                                gpui_component::input::Input::new(
                                    &self.fields.balance_input,
                                ),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Debt;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Debt;
                                    localize(cx, &message)
                                };
                                let error = {
                                    validation_errors
                                        .as_ref()
                                        .and_then(|e| {
                                            let errs = e.debt().all();
                                            if errs.is_empty() {
                                                None
                                            } else {
                                                Some(
                                                    errs
                                                        .iter()
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
                                gpui_component::input::Input::new(&self.fields.debt_input),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::SubscribeNewsletter;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::SubscribeNewsletter;
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
                                gpui_form_collection::switch::Switch::new(
                                    &self.fields.subscribe_newsletter_switch,
                                ),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::EnableNotifications;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::EnableNotifications;
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
                                gpui_form_collection::checkbox::Checkbox::new(
                                    &self.fields.enable_notifications_checkbox,
                                ),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Preferred;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Preferred;
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
                                gpui_component::select::Select::new(
                                    &self.fields.preferred_select,
                                ),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::Country;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::Country;
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
                                gpui_component::select::Select::new(
                                    &self.fields.country_select,
                                ),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = UserLabelVariants::BirthDate;
                                localize(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = UserDescriptionVariants::BirthDate;
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
                                gpui_component::input::Input::new(
                                    &self.fields.birth_date_input,
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
                    "into_original: incomplete; present_fields_json: {}", self
                    .current_data.present_fields_json()
                ),
            )
    }
}
