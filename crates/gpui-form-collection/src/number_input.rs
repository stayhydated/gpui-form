use std::str::FromStr;

use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, IntoElement, RenderOnce, Subscription,
    Window,
};
use gpui_component::input::{
    InputEvent, InputState, NumberInput as GpuiNumberInput,
    NumberInputEvent as GpuiNumberInputEvent, StepAction,
};
use gpui_form_runtime::shape::{ComponentValueBinding, FormValueChange};

#[derive(Clone, Debug)]
pub enum NumberInputEvent {
    Change(String),
}

#[derive(Debug)]
pub struct NumberInputState {
    input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl NumberInputState {
    pub fn new<T>(window: &mut Window, cx: &mut Context<Self>) -> Self
    where
        T: FromStr + ToString + 'static,
    {
        let input = cx
            .new(|cx| InputState::new(window, cx).validate(|value, _| value.parse::<T>().is_ok()));

        let mut subscriptions = Vec::new();

        subscriptions.push(cx.subscribe_in(&input, window, {
            let input = input.clone();
            move |_this, _, event, _window, cx| {
                if let InputEvent::Change = event {
                    cx.emit(NumberInputEvent::Change(input.read(cx).value().to_string()));
                }
            }
        }));

        subscriptions.push(cx.subscribe_in(&input, window, {
            let input = input.clone();
            move |_this, _, event, window, cx| {
                let value = match event {
                    GpuiNumberInputEvent::Step(step) => match step {
                        StepAction::Decrement => {
                            let value = input.read(cx).value().parse::<f64>().unwrap_or(0.0) - 1.;
                            Some(value.to_string())
                        },
                        StepAction::Increment => {
                            let value = input.read(cx).value().parse::<f64>().unwrap_or(0.0) + 1.;
                            Some(value.to_string())
                        },
                    },
                };

                if let Some(value) = value {
                    input.update(cx, |state, cx| {
                        state.set_value(value.as_str(), window, cx);
                    });

                    cx.emit(NumberInputEvent::Change(input.read(cx).value().to_string()));
                }
            }
        }));

        Self {
            input,
            _subscriptions: subscriptions,
        }
    }
}

impl EventEmitter<NumberInputEvent> for NumberInputState {}

#[derive(IntoElement)]
pub struct NumberInputField {
    state: Entity<NumberInputState>,
}

impl NumberInputField {
    pub fn new(state: &Entity<NumberInputState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for NumberInputField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let input = self.state.read(cx).input.clone();
        GpuiNumberInput::new(&input)
    }
}

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_component::input::NumberInput` backed by `InputState`.
    pub struct NumberInput<T = String>
    where
        T: FromStr + ToString + 'static,
    {
        type State = NumberInputState;
        new = |window, cx| NumberInputState::new::<T>(window, cx);
        component = gpui_form_collection::number_input::NumberInputField;
        value_storage = direct;
        field_suffix = "number_input";
        value_binding;

        impl<T> ComponentValueBinding<T> for NumberInput<T>
        where
            T: FromStr + ToString + 'static,
        {
            type Event = NumberInputEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&T>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                let value = value.map(ToString::to_string).unwrap_or_default();

                state.input.update(cx, |state, cx| {
                    state.set_value(value.as_str(), window, cx);
                });
            }

            fn form_value_change(_state: &Self::State, event: &Self::Event) -> FormValueChange<T> {
                match event {
                    NumberInputEvent::Change(value) => {
                        if value.is_empty() {
                            FormValueChange::Clear
                        } else {
                            value
                                .parse()
                                .map_or(FormValueChange::Unchanged, FormValueChange::Set)
                        }
                    },
                }
            }
        }
    }
}
