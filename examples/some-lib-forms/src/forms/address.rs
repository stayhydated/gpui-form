use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyBinding, ParentElement as _, Render, Styled, Subscription, Window, actions,
};
use gpui_component::{
    AxisExt, FocusableCycle, Selectable, Sizable, Size,
    button::{Button, ButtonGroup},
    checkbox::Checkbox,
    date_picker::DatePicker,
    divider::Divider,
    dropdown::{Dropdown, DropdownEvent, DropdownState},
    form::{form_field, v_form},
    h_flex,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction, TextInput},
    switch::Switch,
    v_flex,
};
use rust_decimal::Decimal;
use some_lib::structs::address::*;
use std::sync::{Arc, Mutex};
use story_container::Story;
use story_container::{story, story_init};
actions!(yes_story, [Tab, TabPrev]);
const CONTEXT: &str = "YesForm";
#[story_init]
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("shift-tab", TabPrev, Some(CONTEXT)),
        KeyBinding::new("tab", Tab, Some(CONTEXT)),
    ])
}
#[story]
pub struct AddressForm {
    original_data: Arc<Address>,
    current_data: AddressFormValueHolder,
    fields: AddressFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for AddressForm {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl FocusableCycle for AddressForm {
    fn cycle_focus_handles(&self, _: &mut Window, cx: &mut App) -> Vec<FocusHandle> {
        [
            self.fields.street_input.focus_handle(cx),
            self.fields.dynamic_country_dropdown.focus_handle(cx),
        ]
        .to_vec()
    }
}
impl Story for AddressForm {
    fn title() -> String {
        Address::this_ftl()
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx, Address::default())
    }
}
fn fetch_countries() -> (Vec<Country>, Option<usize>) {
    let c = vec![
        Country::new("United States", "US"),
        Country::new("Canada", "CA"),
        Country::new("Mexico", "MX"),
        Country::new("Brazil", "BR"),
        Country::new("Argentina", "AR"),
        Country::new("Chile", "CL"),
        Country::new("China", "CN"),
        Country::new("Peru", "PE"),
        Country::new("Colombia", "CO"),
        Country::new("Venezuela", "VE"),
        Country::new("Ecuador", "EC"),
    ];
    (c, Some(3))
}

impl AddressForm {
    pub fn view(window: &mut Window, cx: &mut App, original_data: Address) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx, original_data))
    }
    fn tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        self.cycle_focus(true, window, cx);
    }
    fn tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        self.cycle_focus(false, window, cx);
    }
    fn on_street_input_event(
        &mut self,
        _this: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change(text) => {
                self.current_data.street = text.to_owned().into();
            },
            _ => {},
        }
    }
    fn on_dynamic_country_dropdown_event(
        &mut self,
        _this: &Entity<DropdownState<Vec<Country>>>,
        event: &DropdownEvent<Vec<Country>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        match event {
            DropdownEvent::Confirm(value) => {
                if let Some(value) = value {
                    self.current_data.dynamic_country = value.clone().into();
                }
            },
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>, original_data: Address) -> Self {
        let street_input = cx.new(|cx| AddressFormComponents::street_input(window, cx));

        let (countries, country_index) = fetch_countries();
        let dynamic_country_dropdown =
            cx.new(|cx| DropdownState::new(countries, country_index, window, cx));

        let _subscriptions = vec![
            cx.subscribe_in(&street_input, window, Self::on_street_input_event),
            cx.subscribe_in(
                &dynamic_country_dropdown,
                window,
                Self::on_dynamic_country_dropdown_event,
            ),
        ];
        Self {
            original_data: Arc::new(original_data.clone()),
            current_data: original_data.into(),
            fields: AddressFormFields {
                street_input,
                dynamic_country_dropdown,
            },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}
impl Render for AddressForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("yes-form")
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
                            .label(AddressLabelFtl::Street.to_string())
                            .description(AddressDescriptionFtl::Street.to_string())
                            .child(TextInput::new(&self.fields.street_input)),
                    )
                    .child(
                        form_field()
                            .label(AddressLabelFtl::DynamicCountry.to_string())
                            .description(AddressDescriptionFtl::DynamicCountry.to_string())
                            .child(Dropdown::new(&self.fields.dynamic_country_dropdown)),
                    ),
            )
            .child(Divider::horizontal())
            .absolute()
            .child(format!("{:?}", self.current_data))
    }
}
