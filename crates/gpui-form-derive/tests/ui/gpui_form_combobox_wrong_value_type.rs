use gpui_form::GpuiForm;

#[derive(Clone, Debug, PartialEq)]
enum Role {
    Admin,
    User,
}

impl strum::IntoEnumIterator for Role {
    type Iterator = std::array::IntoIter<Role, 2>;

    fn iter() -> Self::Iterator {
        [Role::Admin, Role::User].into_iter()
    }
}

impl gpui_component::select::SelectItem for Role {
    type Value = Self;

    fn title(&self) -> gpui::SharedString {
        match self {
            Self::Admin => "Admin",
            Self::User => "User",
        }
        .into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

#[derive(GpuiForm)]
struct RolesForm {
    #[gpui_form(gpui_form_collection::combobox::Combobox::<_>)]
    roles: Vec<Role>,
}

fn main() {}
