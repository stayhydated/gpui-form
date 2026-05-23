use gpui::{Context, Window};
use gpui_component::{
    IndexPath,
    select::{SelectEvent, SelectItem, SelectState},
};
use gpui_form_component::custom::{CustomComponentValueAdapter, CustomComponentValueChange};
use strum::IntoEnumIterator;

gpui_form_derive::custom_component! {
    /// Shape for a `gpui_component::select::Select` backed by enum variants.
    ///
    /// The enum type `T` must implement `gpui_component::select::SelectItem`,
    /// usually via `#[derive(SelectItem)]` from `gpui-form-collection-derive`.
    pub struct SelectShape<T>
    where
        T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    {
        type State = SelectState<Vec<T>>;
        new = |window, cx| {
            let selected_index = T::iter()
                .position(|item| item == T::default())
                .map(IndexPath::new);
            SelectState::new(T::iter().collect::<Vec<T>>(), selected_index, window, cx)
        };
        component = gpui_component::select::Select;
        value_binding;
    }
}

impl<T> CustomComponentValueAdapter<T> for SelectShape<T>
where
    T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
{
    type Event = SelectEvent<Vec<T>>;

    fn set_state_value(
        state: &mut Self::State,
        value: Option<&T>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        match value {
            Some(value) => state.set_selected_value(value, window, cx),
            None => state.set_selected_index(None, window, cx),
        }
    }

    fn value_change(_state: &Self::State, event: &Self::Event) -> CustomComponentValueChange<T> {
        match event {
            SelectEvent::Confirm(Some(value)) => CustomComponentValueChange::Set(value.clone()),
            SelectEvent::Confirm(None) => CustomComponentValueChange::Clear,
        }
    }
}
