use gpui_kit::component::select::SelectItem as _;
use gpui_form_collection_derive::SelectItem;

#[derive(Clone, PartialEq, SelectItem)]
enum Country {
    UnitedStates,
    Canada,
}

fn main() {
    let _: gpui_kit::SharedString = Country::UnitedStates.title();
    let _: gpui_kit::SharedString = Country::Canada.title();
}
