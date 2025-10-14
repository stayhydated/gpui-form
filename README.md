A struct derive macro for deriving [gpui-component](https://github.com/longbridge/gpui-component)... components on fields.

## Currently Supported components
- Checkbox
- Date Picker
- Dropdown
- Text Input
- Number Input
- Switch

## Usage

GPUI Component and gpui-form are still in development, so you'll need to add them via git.

```toml
gpui = "0.2.0"
gpui-component = { git = "https://github.com/longbridge/gpui-component.git" }
gpui-form = { git = "https://github.com/stayhydated/gpui-form.git" }
```

## Showcase
declaring:
```rs
#[derive(Clone, Debug, Default, DropdownItem, EnumIter, EsFluent, PartialEq)]
#[fluent(display = "std")]
pub enum PreferedLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, DropdownItem, EnumIter, EsFluent, PartialEq)]
#[fluent(display = "std")]
pub enum EnumCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, Default, EsFluentKv, GpuiForm)]
#[fluent_kv(display = "std")]
#[fluent_kv(keys = ["Description", "Label"])]
pub struct User {
    #[gpui_form(component(input))]
    pub username: Option<String>,

    #[gpui_form(component(input))]
    pub email: String,

    #[gpui_form(component(number_input))]
    pub age: Option<u32>,

    #[gpui_form(component(number_input))]
    pub balance: Decimal,

    #[gpui_form(component(checkbox))]
    pub subscribe_newsletter: bool,

    #[gpui_form(component(switch))]
    pub enable_notifications: bool,

    // Signals to use PreferedLanguage::default()
    #[gpui_form(component(dropdown(default)))]
    pub preferred: PreferedLanguage,

    #[gpui_form(component(dropdown(searchable, index = EnumCountry::France)))]
    pub country: Option<EnumCountry>,

    #[gpui_form(component(date_picker))]
    pub birth_date: Option<chrono::NaiveDate>,

    #[gpui_form(skip)]
    pub skip_me: bool,
}
```
this would expand to a structure we normally would have to declare ourselves, reducing boilerplate
```rs
pub struct UserFormValueHolder {
    pub username: String,
    pub email: String,
    pub age: u32,
    pub balance: Decimal,
    pub subscribe_newsletter: bool,
    pub enable_notifications: bool,
    pub preferred: PreferedLanguage,
    pub country: EnumCountry,
    pub birth_date: Option<chrono::NaiveDate>,
    pub skip_me: bool,
}
impl From<User> for UserFormValueHolder
where
    String: ::core::default::Default,
    u32: ::core::default::Default,
    EnumCountry: ::core::default::Default,
    chrono::NaiveDate: ::core::default::Default,
{
    fn from(from: User) -> Self {
        Self {
            username: from.username.unwrap_or_default(),
            email: from.email,
            age: from.age.unwrap_or_default(),
            balance: from.balance,
            subscribe_newsletter: from.subscribe_newsletter,
            enable_notifications: from.enable_notifications,
            preferred: from.preferred,
            country: from.country.unwrap_or_default(),
            birth_date: from.birth_date,
            skip_me: from.skip_me,
        }
    }
}
impl From<UserFormValueHolder> for User {
    fn from(from: UserFormValueHolder) -> Self {
        Self {
            username: Some(from.username),
            email: from.email,
            age: Some(from.age),
            balance: from.balance,
            subscribe_newsletter: from.subscribe_newsletter,
            enable_notifications: from.enable_notifications,
            preferred: from.preferred,
            country: Some(from.country),
            birth_date: from.birth_date,
            skip_me: from.skip_me,
        }
    }
}
impl UserFormValueHolder {
    pub fn try_from(
        from: User,
    ) -> Result<Self, ::gpui_form::unwrapped::UnwrappedError> {
        Ok(Self {
            username: from
                .username
                .ok_or(::gpui_form::unwrapped::UnwrappedError {
                    field_name: "username",
                })?,
            email: from.email,
            age: from
                .age
                .ok_or(::gpui_form::unwrapped::UnwrappedError {
                    field_name: "age",
                })?,
            balance: from.balance,
            subscribe_newsletter: from.subscribe_newsletter,
            enable_notifications: from.enable_notifications,
            preferred: from.preferred,
            country: from
                .country
                .ok_or(::gpui_form::unwrapped::UnwrappedError {
                    field_name: "country",
                })?,
            birth_date: from.birth_date,
            skip_me: from.skip_me,
        })
    }
}
pub struct UserFormFields {
    pub username_input: gpui::Entity<gpui_component::input::InputState>,
    pub email_input: gpui::Entity<gpui_component::input::InputState>,
    pub age_number_input: gpui::Entity<gpui_component::input::InputState>,
    pub balance_number_input: gpui::Entity<gpui_component::input::InputState>,
    pub preferred_dropdown: gpui::Entity<
        gpui_component::dropdown::DropdownState<Vec<PreferedLanguage>>,
    >,
    pub country_dropdown: gpui::Entity<
        gpui_component::dropdown::DropdownState<
            gpui_component::dropdown::SearchableVec<EnumCountry>,
        >,
    >,
    pub birth_date_date_picker: gpui::Entity<
        gpui_component::date_picker::DatePickerState,
    >,
}
pub struct UserFormComponents;
impl UserFormComponents {
    pub fn username_input(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
    ) -> gpui_component::input::InputState {
        gpui_component::input::InputState::new(window, cx)
    }
    pub fn email_input(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
    ) -> gpui_component::input::InputState {
        gpui_component::input::InputState::new(window, cx)
    }
    pub fn age_number_input(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
    ) -> gpui_component::input::InputState {
        use ::gpui_form::NumRegex;
        gpui_component::input::InputState::new(window, cx)
            .pattern(u32::validation_regex().clone())
    }
    pub fn balance_number_input(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
    ) -> gpui_component::input::InputState {
        use ::gpui_form::NumRegex;
        gpui_component::input::InputState::new(window, cx)
            .pattern(Decimal::validation_regex().clone())
    }
    pub fn preferred_dropdown(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<
            '_,
            gpui_component::dropdown::DropdownState<Vec<PreferedLanguage>>,
        >,
    ) -> gpui_component::dropdown::DropdownState<Vec<PreferedLanguage>> {
        use strum::IntoEnumIterator as _;
        gpui_component::dropdown::DropdownState::new(
            PreferedLanguage::iter().collect::<Vec<PreferedLanguage>>().into(),
            Some(
                gpui_component::IndexPath::new(
                    PreferedLanguage::iter()
                        .position(|x| x == PreferedLanguage::default())
                        .unwrap(),
                ),
            ),
            window,
            cx,
        )
    }
    pub fn country_dropdown(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<
            '_,
            gpui_component::dropdown::DropdownState<
                gpui_component::dropdown::SearchableVec<EnumCountry>,
            >,
        >,
    ) -> gpui_component::dropdown::DropdownState<
        gpui_component::dropdown::SearchableVec<EnumCountry>,
    > {
        use strum::IntoEnumIterator as _;
        gpui_component::dropdown::DropdownState::new(
            EnumCountry::iter().collect::<Vec<EnumCountry>>().into(),
            Some(
                gpui_component::IndexPath::new(
                    EnumCountry::iter()
                        .position(|x| x == EnumCountry::France)
                        .unwrap(),
                ),
            ),
            window,
            cx,
        )
    }
    pub fn birth_date_date_picker(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<'_, gpui_component::date_picker::DatePickerState>,
    ) -> gpui_component::date_picker::DatePickerState {
        gpui_component::date_picker::DatePickerState::new(window, cx)
    }
}
```
## Bonus
There's also a prototyping tool which you can customize to your needs (except the [gpui-form-prototyping-core](crates/gpui-form-prototyping-core))

see examples's [README.md](examples/README.md)
