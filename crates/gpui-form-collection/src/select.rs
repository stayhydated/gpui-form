use gpui::{Context, Window};
use gpui_component::{
    IndexPath,
    select::{SelectDelegate, SelectEvent, SelectItem, SelectState},
};
use gpui_form_runtime::shape::{ComponentValueBinding, FormValueChange};
use strum::IntoEnumIterator;

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_component::select::Select` backed by enum variants.
    ///
    /// The enum type `T` must implement `gpui_component::select::SelectItem`,
    /// usually via `#[derive(SelectItem)]` from `gpui-form-collection-derive`.
    pub struct Select<T, D = Vec<T>>
    where
        T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
        D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
    {
        type State = SelectState<D>;
        new = Self::new_default;
        component = gpui_component::select::Select<_>;
        value_storage = direct;
        field_suffix = "select";
        value_binding;

        impl<T, D> ComponentValueBinding<T> for Select<T, D>
        where
            T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
            D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
        {
            type Event = SelectEvent<D>;

            fn seed_value_binding_state(
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

            fn form_value_change(_state: &Self::State, event: &Self::Event) -> FormValueChange<T> {
                match event {
                    SelectEvent::Confirm(Some(value)) => FormValueChange::Set(value.clone()),
                    SelectEvent::Confirm(None) => FormValueChange::Clear,
                }
            }
        }
    }
}

impl<T, D> Select<T, D>
where
    T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
{
    pub fn new_default(
        window: &mut Window,
        cx: &mut Context<'_, SelectState<D>>,
    ) -> SelectState<D> {
        Self::new_with_initial(T::default(), window, cx)
    }

    pub fn new_with_initial(
        initial_value: T,
        window: &mut Window,
        cx: &mut Context<'_, SelectState<D>>,
    ) -> SelectState<D> {
        let selected_index = T::iter()
            .position(|item| item == initial_value)
            .map(IndexPath::new);
        SelectState::new(
            T::iter().collect::<Vec<T>>().into(),
            selected_index,
            window,
            cx,
        )
    }
}
