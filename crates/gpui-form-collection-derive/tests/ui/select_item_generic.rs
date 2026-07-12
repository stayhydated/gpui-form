use gpui_component::select::SelectItem as _;
use gpui_form_collection_derive::SelectItem;

#[derive(Clone, PartialEq, SelectItem)]
enum Choice<T>
where
    T: Clone + PartialEq,
{
    Known(T),
    Empty,
}

fn main() {
    let _: gpui::SharedString = Choice::Known(String::from("stored")).title();
    let _: gpui::SharedString = Choice::<String>::Empty.title();
}
