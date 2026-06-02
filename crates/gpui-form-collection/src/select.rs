use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentShapeBuilder, GpuiComponentValueBinding, component_shape};
use gpui::{Context, Window};
use gpui_component::{
    IndexPath,
    select::{SelectDelegate, SelectEvent, SelectItem, SelectState},
};
use std::marker::PhantomData;
use strum::IntoEnumIterator;

component_shape! {
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
        value = T;
        field_suffix = "select";
        value_binding;

        impl<T, D> GpuiComponentValueBinding<T> for Select<T, D>
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

            fn value_change(_state: &Self::State, event: &Self::Event) -> ValueChange<T> {
                match event {
                    SelectEvent::Confirm(Some(value)) => ValueChange::Set(value.clone()),
                    SelectEvent::Confirm(None) => ValueChange::Clear,
                }
            }
        }
    }
}

impl_form_component_shape!(
    impl<T, D> Select<T, D>
    where [
        T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
        D: SelectDelegate<Item = T> + From<Vec<T>> + 'static
    ];
    gpui_form_runtime::shape::DirectValueStorage
);

impl<T, D> Select<T, D>
where
    T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
{
    pub fn searchable(searchable: bool) -> SelectArgs<T, D> {
        SelectArgs::builder().searchable(searchable).build()
    }

    pub fn from(args: SelectArgs<T, D>) -> SelectArgs<T, D> {
        args
    }

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

#[derive(bon::Builder, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectArgs<T, D = Vec<T>> {
    #[builder(default)]
    searchable: bool,
    #[builder(skip)]
    _marker: PhantomData<fn() -> (T, D)>,
}

impl<T, D> Default for SelectArgs<T, D> {
    fn default() -> Self {
        Self {
            searchable: false,
            _marker: PhantomData,
        }
    }
}

impl<T, D> GpuiComponentShapeBuilder<Select<T, D>> for SelectArgs<T, D>
where
    T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
{
    fn build(self, window: &mut Window, cx: &mut Context<'_, SelectState<D>>) -> SelectState<D> {
        Select::<T, D>::new_default(window, cx).searchable(self.searchable)
    }
}

#[cfg(test)]
mod tests {
    use super::{Select, SelectArgs};
    use component_shape_gpui::GpuiComponentShapeBuilder;
    use gpui_component::select::{SelectDelegate, SelectItem};
    use strum::IntoEnumIterator;

    #[test]
    fn select_args_default_to_non_searchable() {
        assert!(!SelectArgs::<()>::default().searchable);
        assert!(!SelectArgs::<()>::builder().build().searchable);
    }

    #[test]
    fn select_args_record_searchable_configuration() {
        assert!(
            SelectArgs::<()>::builder()
                .searchable(true)
                .build()
                .searchable
        );
    }

    #[allow(dead_code)]
    fn assert_select_args_build_shape<T, D>() -> (SelectArgs<T, D>, SelectArgs<T, D>)
    where
        T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
        D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
        SelectArgs<T, D>: GpuiComponentShapeBuilder<Select<T, D>>,
    {
        (
            Select::<T, D>::from(SelectArgs::<T, D>::builder().searchable(true).build()),
            Select::<T, D>::searchable(true),
        )
    }
}
