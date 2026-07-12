use std::marker::PhantomData;

struct SelectState<T> {
    searchable: bool,
    _marker: PhantomData<T>,
}

impl<T> SelectState<T> {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self {
            searchable: false,
            _marker: PhantomData,
        }
    }
}

component_shape_gpui::component_shape! {
    struct Select<T>
    where
        T: 'static,
    {
        state = SelectState<T>;
        value = T;
    }
}

impl<T> gpui_form_runtime::shape::GpuiFormComponentShapePolicy for Select<T>
where
    T: 'static,
{
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(gpui_form::bon::Builder)]
#[builder(crate = ::gpui_form::bon)]
struct SelectArgs<T> {
    #[builder(default)]
    searchable: bool,
    #[builder(skip)]
    _marker: PhantomData<T>,
}

impl<T> Select<T> {
    fn searchable(searchable: bool) -> SelectArgs<T> {
        SelectArgs::builder().searchable(searchable).build()
    }

    fn from(args: SelectArgs<T>) -> SelectArgs<T> {
        args
    }
}

impl<T> gpui_form_runtime::shape::GpuiComponentShapeBuilder<Select<T>> for SelectArgs<T>
where
    T: 'static,
{
    fn build(
        self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, SelectState<T>>,
    ) -> SelectState<T> {
        SelectState {
            searchable: self.searchable,
            _marker: PhantomData,
        }
    }
}

#[derive(gpui_form::GpuiForm)]
#[gpui_form(no_inventory)]
struct ConfiguredComponentForm {
    #[gpui_form(component(Select::<_>.searchable(true)))]
    country: String,
    #[gpui_form(component(Select::<_>.searchable(false)))]
    city: String,
    #[gpui_form(component(Select::<_>.from(
        SelectArgs::builder().searchable(true).build()
    )))]
    region: String,
}

fn accepts_components(_: ConfiguredComponentFormFormComponents) {}

fn main() {
    let _ = SelectState::<String> {
        searchable: false,
        _marker: PhantomData,
    }
    .searchable;
    let _ = accepts_components;
}
