use gpui::{Context, Window};
use gpui_component::{
    IndexPath,
    select::{SelectDelegate, SelectEvent, SelectItem, SelectState},
};
use gpui_form_component::custom::{CustomComponentValueBinding, FormValueEvent};
use strum::IntoEnumIterator;

gpui_form_derive::custom_component! {
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
        component = gpui_component::select::Select;
        value_binding;
        field_suffix = "select";
    }
}

/// Options used by `#[gpui_form(component = Select::<_>::searchable(...)...)]`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectOptions {
    pub searchable: bool,
    pub partial: bool,
}

#[bon::bon]
impl<T, D> Select<T, D>
where
    T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
{
    /// Starts a bon-style `#[gpui_form(component = ...)]` option chain.
    #[builder(start_fn = builder, finish_fn = build)]
    pub fn options(
        #[builder(default)] searchable: bool,
        #[builder(default)] partial: bool,
    ) -> SelectOptions {
        SelectOptions {
            searchable,
            partial,
        }
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

#[allow(unnameable_types)]
impl<T, D> Select<T, D>
where
    T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
{
    /// Starts a `#[gpui_form(component = ...)]` option chain with search enabled.
    pub fn searchable(
        value: bool,
    ) -> SelectOptionsBuilder<T, D, select_options_builder::SetSearchable> {
        Self::builder().searchable(value)
    }

    /// Starts a `#[gpui_form(component = ...)]` option chain with partial rendering enabled.
    pub fn partial(value: bool) -> SelectOptionsBuilder<T, D, select_options_builder::SetPartial> {
        Self::builder().partial(value)
    }
}

impl<T, D> CustomComponentValueBinding<T> for Select<T, D>
where
    T: Clone + Default + IntoEnumIterator + PartialEq + SelectItem<Value = T> + 'static,
    D: SelectDelegate<Item = T> + From<Vec<T>> + 'static,
{
    type NativeEvent = SelectEvent<D>;

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

    fn form_value_event(_state: &Self::State, event: &Self::NativeEvent) -> FormValueEvent<T> {
        match event {
            SelectEvent::Confirm(Some(value)) => FormValueEvent::Change(value.clone()),
            SelectEvent::Confirm(None) => FormValueEvent::Clear,
        }
    }
}
