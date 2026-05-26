use gpui::{Context, Window};
use gpui_component::{
    combobox::{ComboboxEvent, ComboboxState},
    searchable_list::SearchableListDelegate,
    select::SelectItem,
    IndexPath,
};
use gpui_form_component::shape::{ComponentValueBinding, FormValueChange};
use strum::IntoEnumIterator;

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_component::combobox::Combobox` backed by `ComboboxState`.
    ///
    /// The enum type `T` must implement `gpui_component::select::SelectItem`,
    /// usually via `#[derive(SelectItem)]` from `gpui-form-collection-derive`.
    pub struct Combobox<T, D = Vec<T>>
    where
        T: Clone + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
        D: SearchableListDelegate<Item = T> + From<Vec<T>> + 'static,
    {
        type State = ComboboxState<D>;
        new = Self::new_default;
        component = gpui_component::combobox::Combobox<D>;
        value_binding;
        field_suffix = "combobox";
    }
}

impl<T, D> Combobox<T, D>
where
    T: Clone + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SearchableListDelegate<Item = T> + From<Vec<T>> + 'static,
{
    pub fn new_default(
        window: &mut Window,
        cx: &mut Context<'_, ComboboxState<D>>,
    ) -> ComboboxState<D> {
        Self::new_with_initial(&[], window, cx)
    }

    pub fn new_with_initial(
        value: &[T],
        window: &mut Window,
        cx: &mut Context<'_, ComboboxState<D>>,
    ) -> ComboboxState<D> {
        let all_items = T::iter().collect::<Vec<T>>();
        let selected_indices = value
            .iter()
            .filter_map(|value| {
                all_items
                    .iter()
                    .position(|candidate| candidate == value)
                    .map(IndexPath::new)
            })
            .collect::<Vec<_>>();

        ComboboxState::new(all_items.into(), selected_indices, window, cx)
    }
}

impl<T, D> ComponentValueBinding<Vec<T>> for Combobox<T, D>
where
    T: Clone + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SearchableListDelegate<Item = T> + From<Vec<T>> + 'static,
{
    type Event = ComboboxEvent<D>;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&Vec<T>>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        let value = value.map_or(&[] as &[T], |value| value.as_slice());
        let all_items = T::iter().collect::<Vec<T>>();
        let selected_indices = value
            .iter()
            .filter_map(|value| {
                all_items
                    .iter()
                    .position(|candidate| candidate == value)
                    .map(IndexPath::new)
            })
            .collect::<Vec<_>>();

        state.set_selected_indices(selected_indices, window, cx);
    }

    fn form_value_change(
        _state: &Self::State,
        event: &Self::Event,
    ) -> FormValueChange<Vec<T>> {
        match event {
            ComboboxEvent::Change(values) | ComboboxEvent::Confirm(values) => {
                if values.is_empty() {
                    FormValueChange::Clear
                } else {
                    FormValueChange::Set(values.clone())
                }
            }
        }
    }
}
