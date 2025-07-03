use some_lib::structs::user::*;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, KeyBinding, ParentElement as _, Render, Styled, Subscription, Window,
    actions,
};
use gpui_component::{
    AxisExt, FocusableCycle, Selectable, Sizable, Size, button::{Button, ButtonGroup},
    checkbox::Checkbox, date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    divider::Divider,
    dropdown::{Dropdown, DropdownEvent, DropdownItem, DropdownState, SearchableVec},
    form::{form_field, v_form},
    h_flex,
    input::{
        InputEvent, InputState, NumberInput, NumberInputEvent, StepAction, TextInput,
    },
    switch::Switch, v_flex,
};
use rust_decimal::Decimal;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use story_container::story::Story;
actions!(user_story, [Tab, TabPrev]);
const CONTEXT: &str = "UserForm";
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("shift-tab", TabPrev, Some(CONTEXT)),
        KeyBinding::new("tab", Tab, Some(CONTEXT)),
    ])
}
pub struct UserForm {
    original_data: Arc<User>,
    current_data: UserFormValueHolder,
    fields: UserFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for UserForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl FocusableCycle for UserForm {
    fn cycle_focus_handles(&self, _: &mut Window, cx: &mut App) -> Vec<FocusHandle> {
        [
            self.fields.username_input.focus_handle(cx),
            self.fields.email_input.focus_handle(cx),
            self.fields.age_number_input.focus_handle(cx),
            self.fields.balance_number_input.focus_handle(cx),
            self.fields.preferred_dropdown.focus_handle(cx),
            self.fields.country_dropdown.focus_handle(cx),
            self.fields.birth_date_date_picker.focus_handle(cx),
        ]
            .to_vec()
    }
}
impl Story for UserForm {
    fn title() -> &'static str {
        "User"
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx, User::default())
    }
}
impl UserForm {
    pub fn view(window: &mut Window, cx: &mut App, original_data: User) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, original_data))
    }
    fn tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        self.cycle_focus(true, window, cx);
    }
    fn tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        self.cycle_focus(false, window, cx);
    }
    fn on_username_input_event(
        &mut self,
        _this: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change(text) => {
                self.current_data.username = text.to_owned().into();
            }
            _ => {}
        }
    }
    fn on_email_input_event(
        &mut self,
        _this: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change(text) => {
                self.current_data.email = text.to_owned().into();
            }
            _ => {}
        }
    }
    fn on_age_input_event(
        &mut self,
        _this: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change(text) => {
                if let Ok(value) = text.parse::<u32>() {
                    self.current_data.age = value.into();
                }
            }
            _ => {}
        }
    }
    fn on_age_number_input_event(
        &mut self,
        this: &Entity<InputState>,
        event: &NumberInputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NumberInputEvent::Step(step_action) => {
                match step_action {
                    StepAction::Decrement => {
                        let new_value = self.current_data.age.saturating_sub(1 as u32);
                        self.current_data.age = new_value;
                        this.update(
                            cx,
                            |input, cx| {
                                input
                                    .set_value(self.current_data.age.to_string(), window, cx);
                            },
                        );
                    }
                    StepAction::Increment => {
                        let new_value = self.current_data.age.saturating_add(1 as u32);
                        self.current_data.age = new_value;
                        this.update(
                            cx,
                            |input, cx| {
                                input
                                    .set_value(self.current_data.age.to_string(), window, cx);
                            },
                        );
                    }
                }
            }
        }
    }
    fn on_balance_input_event(
        &mut self,
        _this: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change(text) => {
                if let Ok(value) = text.parse::<Decimal>() {
                    self.current_data.balance = value.into();
                }
            }
            _ => {}
        }
    }
    fn on_balance_number_input_event(
        &mut self,
        this: &Entity<InputState>,
        event: &NumberInputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NumberInputEvent::Step(step_action) => {
                match step_action {
                    StepAction::Decrement => {
                        let new_value = self
                            .current_data
                            .balance
                            .saturating_sub(Decimal::from(1));
                        self.current_data.balance = new_value;
                        this.update(
                            cx,
                            |input, cx| {
                                input
                                    .set_value(
                                        self.current_data.balance.to_string(),
                                        window,
                                        cx,
                                    );
                            },
                        );
                    }
                    StepAction::Increment => {
                        let new_value = self
                            .current_data
                            .balance
                            .saturating_add(Decimal::from(1));
                        self.current_data.balance = new_value;
                        this.update(
                            cx,
                            |input, cx| {
                                input
                                    .set_value(
                                        self.current_data.balance.to_string(),
                                        window,
                                        cx,
                                    );
                            },
                        );
                    }
                }
            }
        }
    }
    fn on_preferred_dropdown_event(
        &mut self,
        _this: &Entity<DropdownState<Vec<PreferedLanguage>>>,
        event: &DropdownEvent<Vec<PreferedLanguage>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            DropdownEvent::Confirm(value) => {
                if let Some(value) = value {
                    self.current_data.preferred = value.clone().into();
                }
            }
        }
    }
    fn on_country_dropdown_event(
        &mut self,
        _this: &Entity<DropdownState<SearchableVec<EnumCountry>>>,
        event: &DropdownEvent<SearchableVec<EnumCountry>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            DropdownEvent::Confirm(value) => {
                if let Some(value) = value {
                    self.current_data.country = value.clone().into();
                }
            }
        }
    }
    fn on_birth_date_date_picker_event(
        &mut self,
        _this: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            DatePickerEvent::Change(date) => {
                self.current_data.birth_date = chrono::NaiveDate::parse_from_str(
                        &date.to_owned().to_string(),
                        "%Y-%m-%d",
                    )
                    .ok();
            }
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>, original_data: User) -> Self {
        let username_input = cx.new(|cx| UserFormComponents::username_input(window, cx));
        let email_input = cx.new(|cx| UserFormComponents::email_input(window, cx));
        let age_number_input = cx
            .new(|cx| UserFormComponents::age_number_input(window, cx));
        let balance_number_input = cx
            .new(|cx| UserFormComponents::balance_number_input(window, cx));
        let preferred_dropdown = cx
            .new(|cx| UserFormComponents::preferred_dropdown(window, cx));
        let country_dropdown = cx
            .new(|cx| UserFormComponents::country_dropdown(window, cx));
        let birth_date_date_picker = cx
            .new(|cx| UserFormComponents::birth_date_date_picker(window, cx));
        let _subscriptions = vec![
            cx.subscribe_in(& username_input, window, Self::on_username_input_event), cx
            .subscribe_in(& email_input, window, Self::on_email_input_event), cx
            .subscribe_in(& age_number_input, window, Self::on_age_input_event), cx
            .subscribe_in(& age_number_input, window, Self::on_age_number_input_event),
            cx.subscribe_in(& balance_number_input, window,
            Self::on_balance_input_event), cx.subscribe_in(& balance_number_input,
            window, Self::on_balance_number_input_event), cx.subscribe_in(&
            preferred_dropdown, window, Self::on_preferred_dropdown_event), cx
            .subscribe_in(& country_dropdown, window, Self::on_country_dropdown_event),
            cx.subscribe_in(& birth_date_date_picker, window,
            Self::on_birth_date_date_picker_event)
        ];
        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            fields: UserFormFields {
                username_input,
                email_input,
                age_number_input,
                balance_number_input,
                preferred_dropdown,
                country_dropdown,
                birth_date_date_picker,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}
impl Render for UserForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("user-form")
            .on_action(cx.listener(Self::tab))
            .on_action(cx.listener(Self::tab_prev))
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Divider::horizontal())
            .child(
                v_form()
                    .child(
                        form_field()
                            .label(UserLabelFtl::Username.to_string())
                            .description(UserDescriptionFtl::Username.to_string())
                            .child(TextInput::new(&self.fields.username_input)),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::Email.to_string())
                            .description(UserDescriptionFtl::Email.to_string())
                            .child(TextInput::new(&self.fields.email_input)),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::Age.to_string())
                            .description(UserDescriptionFtl::Age.to_string())
                            .child(NumberInput::new(&self.fields.age_number_input)),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::Balance.to_string())
                            .description(UserDescriptionFtl::Balance.to_string())
                            .child(NumberInput::new(&self.fields.balance_number_input)),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::SubscribeNewsletter.to_string())
                            .description(
                                UserDescriptionFtl::SubscribeNewsletter.to_string(),
                            )
                            .child(
                                Checkbox::new("subscribe-newsletter-checkbox")
                                    .checked(self.current_data.subscribe_newsletter)
                                    .on_click(
                                        cx
                                            .listener(|v, _, _, _| {
                                                v.current_data.subscribe_newsletter = !v
                                                    .current_data
                                                    .subscribe_newsletter;
                                            }),
                                    ),
                            ),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::EnableNotifications.to_string())
                            .description(
                                UserDescriptionFtl::EnableNotifications.to_string(),
                            )
                            .child(
                                Switch::new("enable-notifications-switch")
                                    .checked(self.current_data.enable_notifications)
                                    .on_click(
                                        cx
                                            .listener(move |v, checked, _, cx| {
                                                v.current_data.enable_notifications = *checked;
                                                cx.notify();
                                            }),
                                    ),
                            ),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::Preferred.to_string())
                            .description(UserDescriptionFtl::Preferred.to_string())
                            .child(Dropdown::new(&self.fields.preferred_dropdown)),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::Country.to_string())
                            .description(UserDescriptionFtl::Country.to_string())
                            .child(Dropdown::new(&self.fields.country_dropdown)),
                    )
                    .child(
                        form_field()
                            .label(UserLabelFtl::BirthDate.to_string())
                            .description(UserDescriptionFtl::BirthDate.to_string())
                            .child(DatePicker::new(&self.fields.birth_date_date_picker)),
                    ),
            )
            .child(Divider::horizontal())
            .absolute()
            .child(format!("{:?}", self.current_data))
    }
}
