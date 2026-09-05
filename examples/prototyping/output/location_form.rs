use gpui_kit::{
    App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window,
};
use gpui_kit::{InteractiveElement as _, ParentElement as _, Styled as _, Subscription, div};
use gpui_kit::component::Disableable as _;
use gpui_kit::component::form::field;
use gpui_kit::component::form::v_form;
use gpui_kit::component::separator::Separator;
use gpui_kit::component::v_flex;
use gpui_form::runtime::shape::{
    GpuiComponentEventOf, GpuiComponentStateOf, ValueChange, seed_value_binding_state, value_change,
};
use some_lib::structs::form_action::FormAction;
use some_lib::structs::location::*;
const CONTEXT: &str = "LocationFormForm";
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[gpui_storybook::story]
#[derive(gpui_storybook::StoryControls)]
pub struct LocationFormForm {
    current_data: LocationFormFormValueHolder,
    fields: LocationFormFormFields,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}
impl Focusable for LocationFormForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for LocationFormForm {
    fn title(cx: &gpui_kit::App) -> String {
        gpui_es_fluent::localize_label::<LocationForm>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl LocationFormForm {
    fn on_name_input_event(
        &mut self,
        state: &Entity<GpuiComponentStateOf<gpui_form_collection::input::Input<String>>>,
        event: &GpuiComponentEventOf<gpui_form_collection::input::Input<String>, String>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_collection::input::Input<String>, String>(state, event)
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.name = value;
            },
            ValueChange::Clear => {
                self.current_data.name = <<gpui_form_collection::input::Input<
                    String,
                > as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    String,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn on_location_infinite_select_event(
        &mut self,
        state: &Entity<
            GpuiComponentStateOf<gpui_form_component::infinite_select::InfiniteSelect<Country>>,
        >,
        event: &GpuiComponentEventOf<
            gpui_form_component::infinite_select::InfiniteSelect<Country>,
            Country,
        >,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        let form_change = {
            let state = state.read(_cx);
            value_change::<gpui_form_component::infinite_select::InfiniteSelect<Country>, Country>(
                state, event,
            )
        };
        match form_change {
            ValueChange::Set(value) => {
                self.current_data.location = value;
            },
            ValueChange::Clear => {
                self.current_data.location = <<gpui_form_component::infinite_select::InfiniteSelect<
                    Country,
                > as gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy as gpui_form::runtime::shape::DefaultValueStorage<
                    Country,
                >>::default_storage();
            },
            ValueChange::Unchanged => {},
        }
    }
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_data = LocationFormFormValueHolder::default();
        let name = cx.new(|cx| LocationFormFormComponents::name(window, cx));
        let location = cx.new(|cx| LocationFormFormComponents::location(window, cx));
        let mut _subscriptions = vec![
            cx.subscribe_in(&name, window, Self::on_name_input_event),
            cx.subscribe_in(&location, window, Self::on_location_infinite_select_event),
        ];
        name.update(cx, |state, cx| {
            seed_value_binding_state::<gpui_form_collection::input::Input<String>, String>(
                state,
                Some(&current_data.name),
                window,
                cx,
            );
        });
        location.update(cx, |state, cx| {
            seed_value_binding_state::<
                gpui_form_component::infinite_select::InfiniteSelect<Country>,
                Country,
            >(state, Some(&current_data.location), window, cx);
        });
        Self {
            current_data,
            fields: LocationFormFormFields { name, location },
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
    fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        *self = Self::new(window, cx);
        cx.notify();
    }
    fn submit_payload(&self) -> Option<LocationForm> {
        self.current_data.clone().try_into_original().ok()
    }
    fn submit_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui_kit::SharedString>,
        on_submit: impl Fn(Option<LocationForm>, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui_kit::component::button::Button {
        gpui_kit::component::button::Button::new(format!("{}-submit-button", "location_form-form"))
            .label(label)
            .disabled(false)
            .on_click(cx.listener(move |this, _, window, cx| {
                on_submit(this.submit_payload(), window, cx);
            }))
    }
    fn reset_button(
        &self,
        cx: &mut Context<Self>,
        label: impl Into<gpui_kit::SharedString>,
    ) -> gpui_kit::component::button::Button {
        gpui_kit::component::button::Button::new(format!("{}-reset-button", "location_form-form"))
            .label(label)
            .on_click(cx.listener(|this, _, window, cx| {
                this.reset_form(window, cx);
            }))
    }
    fn action_buttons(
        &self,
        cx: &mut Context<Self>,
        on_submit: impl Fn(Option<LocationForm>, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(self.submit_button(
                cx,
                gpui_es_fluent::localize_message(cx, &FormAction::Submit),
                on_submit,
            ))
            .child(self.reset_button(cx, gpui_es_fluent::localize_message(cx, &FormAction::Reset)))
    }
}
impl Render for LocationFormForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("location_form-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Separator::horizontal())
            .child(
                v_form()
                    .child(
                        field()
                            .label({
                                let message = LocationFormLabelVariants::Name;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = LocationFormDescriptionVariants::Name;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(
                                <<gpui_form_collection::input::Input<
                                    String,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_collection::input::Input<
                                        String,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.name),
                            ),
                    )
                    .child(
                        field()
                            .label({
                                let message = LocationFormLabelVariants::Location;
                                gpui_es_fluent::localize_message(cx, &message)
                            })
                            .description_fn({
                                let description = {
                                    let message = LocationFormDescriptionVariants::Location;
                                    gpui_es_fluent::localize_message(cx, &message)
                                };
                                move |_, _| {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().child(description.clone()))
                                }
                            })
                            .child(
                                <<gpui_form_component::infinite_select::InfiniteSelect<
                                    Country,
                                > as gpui_form::runtime::shape::GpuiComponentShape>::RenderComponent as gpui_form::runtime::shape::GpuiComponentRender<
                                    <gpui_form_component::infinite_select::InfiniteSelect<
                                        Country,
                                    > as gpui_form::runtime::shape::GpuiComponentShape>::State,
                                >>::new(&self.fields.location),
                            ),
                    )
                    .child(
                        field()
                            .label_indent(false)
                            .child(
                                self
                                    .action_buttons(
                                        cx,
                                        |payload, _, _| {
                                            let _ = payload;
                                        },
                                    ),
                            ),
                    ),
            )
            .child(Separator::horizontal())
            .child(format!("value_holder: {:?}", self.current_data))
            .child(
                format!(
                    "try_into_original: {:?}", self.current_data.clone()
                    .try_into_original()
                ),
            )
    }
}
