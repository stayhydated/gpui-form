//! Curated component shapes for `gpui-form`.

extern crate self as gpui_form_collection;

macro_rules! impl_form_component_shape {
    (impl<$($generics:ident),+> $shape:ty where [$($where_clause:tt)*]; $storage:ty) => {
        impl<$($generics),+> gpui_form_runtime::shape::GpuiFormComponentShapePolicy for $shape
        where
            $($where_clause)*
        {
            type ValueStoragePolicy = $storage;
        }
    };
    ($shape:ty, $storage:ty) => {
        impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for $shape {
            type ValueStoragePolicy = $storage;
        }
    };
}

pub mod checkbox;
pub mod color_picker;
pub mod combobox;
pub mod date_picker;
pub mod input;
pub mod number_input;
pub mod otp_input;
pub mod select;
pub mod slider;
pub mod switch;

#[cfg(test)]
mod tests {
    use gpui_kit as gpui;

    use super::{
        checkbox::{Checkbox, CheckboxEvent, CheckboxField, CheckboxState},
        color_picker::ColorPicker,
        combobox::Combobox,
        date_picker::{DatePicker, DateRangePicker},
        input::Input,
        number_input::{NumberInput, NumberInputEvent, NumberInputField, NumberInputState},
        otp_input::OtpInput,
        slider::Slider,
        switch::{Switch, SwitchEvent, SwitchField, SwitchState},
    };
    use chrono::NaiveDate;
    use component_shape::{DeclaredComponentShape, ValueChange};
    use gpui_form_runtime::shape::{
        GpuiComponentEventOf, GpuiComponentShape, GpuiComponentShapeFor, GpuiComponentStateOf,
        GpuiComponentValueBinding, GpuiFormComponentShapePolicy,
    };
    use gpui_kit::component::{
        combobox::{ComboboxEvent, ComboboxState},
        input::{OtpEvent, OtpState},
        select::SelectItem,
        slider::SliderValue,
    };
    use gpui_kit::{
        AppContext as _, Context, Empty, Entity, EventEmitter, Hsla, IntoElement, Render,
        SharedString, TestAppContext, Window,
    };
    use strum::IntoEnumIterator;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Choice {
        First,
        Second,
    }

    impl IntoEnumIterator for Choice {
        type Iterator = std::array::IntoIter<Self, 2>;

        fn iter() -> Self::Iterator {
            [Self::First, Self::Second].into_iter()
        }
    }

    impl SelectItem for Choice {
        type Value = Self;

        fn title(&self) -> SharedString {
            match self {
                Self::First => "First",
                Self::Second => "Second",
            }
            .into()
        }

        fn value(&self) -> &Self::Value {
            self
        }
    }

    struct StateHarness {
        checkbox: Entity<CheckboxState>,
        combobox: Entity<ComboboxState<Vec<Choice>>>,
        number: Entity<NumberInputState>,
        otp: Entity<OtpState>,
        switch: Entity<SwitchState>,
    }

    impl Render for StateHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            Empty
        }
    }

    fn assert_declared_shape<Shape>()
    where
        Shape: DeclaredComponentShape + GpuiComponentShape + GpuiFormComponentShapePolicy,
    {
    }

    fn assert_shape_for<Shape, Value>()
    where
        Shape: GpuiComponentShapeFor<Value>,
    {
    }

    fn assert_value_binding<Shape, Value>()
    where
        Shape: GpuiComponentValueBinding<Value>,
        GpuiComponentStateOf<Shape>: EventEmitter<GpuiComponentEventOf<Shape, Value>>,
    {
    }

    #[test]
    fn built_in_components_publish_component_shape_contracts() {
        assert_declared_shape::<Checkbox>();
        assert_shape_for::<Checkbox, bool>();
        assert_value_binding::<Checkbox, bool>();

        assert_declared_shape::<Switch>();
        assert_shape_for::<Switch, bool>();
        assert_value_binding::<Switch, bool>();

        assert_declared_shape::<ColorPicker>();
        assert_shape_for::<ColorPicker, Hsla>();
        assert_value_binding::<ColorPicker, Hsla>();

        assert_declared_shape::<DatePicker>();
        assert_shape_for::<DatePicker, NaiveDate>();
        assert_value_binding::<DatePicker, NaiveDate>();

        assert_declared_shape::<DateRangePicker>();
        assert_shape_for::<DateRangePicker, (NaiveDate, NaiveDate)>();
        assert_value_binding::<DateRangePicker, (NaiveDate, NaiveDate)>();

        assert_declared_shape::<Input<String>>();
        assert_shape_for::<Input<String>, String>();
        assert_value_binding::<Input<String>, String>();
        assert_shape_for::<Input<u32>, u32>();

        assert_declared_shape::<NumberInput<u32>>();
        assert_shape_for::<NumberInput<u32>, u32>();
        assert_value_binding::<NumberInput<u32>, u32>();

        assert_declared_shape::<OtpInput<String>>();
        assert_shape_for::<OtpInput<String>, String>();
        assert_value_binding::<OtpInput<String>, String>();

        assert_declared_shape::<Slider>();
        assert_shape_for::<Slider, SliderValue>();
        assert_shape_for::<Slider, f32>();
        assert_value_binding::<Slider, SliderValue>();
        assert_value_binding::<Slider, f32>();
    }

    #[gpui_kit::test]
    fn gpui_state_constructors_and_value_bindings_round_trip_without_rendering(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_kit::component::init);
        let window = cx.add_window(|window, cx| StateHarness {
            checkbox: cx.new(|cx| <Checkbox as GpuiComponentShape>::new(window, cx)),
            combobox: cx.new(|cx| <Combobox<Choice> as GpuiComponentShape>::new(window, cx)),
            number: cx.new(|cx| <NumberInput<i32> as GpuiComponentShape>::new(window, cx)),
            otp: cx.new(|cx| <OtpInput<i32> as GpuiComponentShape>::new(window, cx)),
            switch: cx.new(|cx| <Switch as GpuiComponentShape>::new(window, cx)),
        });

        window
            .update(cx, |harness, window, cx| {
                harness.checkbox.update(cx, |state, cx| {
                    <Checkbox as GpuiComponentValueBinding<bool>>::seed_value_binding_state(
                        state,
                        Some(&true),
                        window,
                        cx,
                    );
                    state.set_checked(true, cx);
                });
                harness.switch.update(cx, |state, cx| {
                    <Switch as GpuiComponentValueBinding<bool>>::seed_value_binding_state(
                        state,
                        Some(&true),
                        window,
                        cx,
                    );
                    state.set_checked(true, cx);
                });
                harness.combobox.update(cx, |state, cx| {
                    <Combobox<Choice> as GpuiComponentValueBinding<Vec<Choice>>>::seed_value_binding_state(
                        state,
                        Some(&vec![Choice::Second]),
                        window,
                        cx,
                    );
                });
                harness.number.update(cx, |state, cx| {
                    <NumberInput<i32> as GpuiComponentValueBinding<i32>>::seed_value_binding_state(
                        state,
                        Some(&42),
                        window,
                        cx,
                    );
                });
                harness.otp.update(cx, |state, cx| {
                    <OtpInput<i32> as GpuiComponentValueBinding<i32>>::seed_value_binding_state(
                        state,
                        Some(&123_456),
                        window,
                        cx,
                    );
                });
            })
            .unwrap();

        let harness = window.root(cx).unwrap();
        let checkbox = harness.read_with(cx, |harness, _| harness.checkbox.clone());
        let switch = harness.read_with(cx, |harness, _| harness.switch.clone());
        let combobox = harness.read_with(cx, |harness, _| harness.combobox.clone());
        let number = harness.read_with(cx, |harness, _| harness.number.clone());
        let otp = harness.read_with(cx, |harness, _| harness.otp.clone());

        assert!(checkbox.read_with(cx, |state, _| state.checked()));
        assert!(switch.read_with(cx, |state, _| state.checked()));
        assert_eq!(
            combobox.read_with(cx, |state, _| state.selected_values()),
            vec![Choice::Second]
        );
        assert_eq!(
            otp.read_with(cx, |state, _| state.value().to_string()),
            "123456"
        );

        assert_eq!(
            checkbox.read_with(cx, |state, _| {
                <Checkbox as GpuiComponentValueBinding<bool>>::value_change(
                    state,
                    &CheckboxEvent::Change(false),
                )
            }),
            ValueChange::Set(false)
        );
        assert_eq!(
            switch.read_with(cx, |state, _| {
                <Switch as GpuiComponentValueBinding<bool>>::value_change(
                    state,
                    &SwitchEvent::Change(false),
                )
            }),
            ValueChange::Set(false)
        );
        assert_eq!(
            combobox.read_with(cx, |state, _| {
                <Combobox<Choice> as GpuiComponentValueBinding<Vec<Choice>>>::value_change(
                    state,
                    &ComboboxEvent::Confirm(vec![Choice::First]),
                )
            }),
            ValueChange::Set(vec![Choice::First])
        );
        assert_eq!(
            number.read_with(cx, |state, _| {
                <NumberInput<i32> as GpuiComponentValueBinding<i32>>::value_change(
                    state,
                    &NumberInputEvent::Change("7".to_owned()),
                )
            }),
            ValueChange::Set(7)
        );
        assert_eq!(
            otp.read_with(cx, |state, _| {
                <OtpInput<i32> as GpuiComponentValueBinding<i32>>::value_change(
                    state,
                    &OtpEvent::Change,
                )
            }),
            ValueChange::Set(123_456)
        );
        assert_eq!(
            otp.read_with(cx, |state, _| {
                <OtpInput<i32> as GpuiComponentValueBinding<i32>>::value_change(
                    state,
                    &OtpEvent::Blur,
                )
            }),
            ValueChange::Unchanged
        );

        let _ = CheckboxField::new(&checkbox);
        let _ = SwitchField::new(&switch);
        let _ = NumberInputField::new(&number);
        let _ = super::otp_input::OtpInputField::new(&otp);
    }
}
