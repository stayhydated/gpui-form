use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use strum::EnumIter;

es_fluent_manager_embedded::define_i18n_module!();

pub use gpui_es_fluent::{
    EmbeddedI18n, EmbeddedInitError, I18n, LocalizationError, change_locale, localize_label,
    localize_message, try_localize_label, try_localize_message,
};

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, Eq, EsFluent, PartialEq)]
pub enum Languages {}

/// Applies Storybook's resolved locale to the component gallery's GPUI manager.
pub fn apply_locale(
    language: Languages,
    cx: &mut gpui_kit::App,
) -> Result<(), gpui_es_fluent::EmbeddedInitError> {
    let _linked_module = &GPUI_FORM_COMPONENT_STORY_I18N_MODULE;
    gpui_es_fluent::replace_with_language(cx, language)
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "date_picker")]
pub(crate) enum DatePickerComponentText {
    LaunchPlaceholder,
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "file_picker")]
pub(crate) enum FilePickerComponentText {
    SourcePlaceholder,
    OutputPlaceholder,
    ChooseFiles,
}

#[cfg(test)]
mod tests {
    use super::{DatePickerComponentText, Languages, apply_locale};
    use gpui_kit as gpui;

    #[gpui_kit::test]
    fn gpui_adapter_links_and_applies_component_resources(cx: &mut gpui_kit::TestAppContext) {
        cx.update(|cx| {
            apply_locale(Languages::FrFr, cx)
                .expect("French component story resources should initialize");
            assert_eq!(
                gpui_es_fluent::localize_message(cx, &DatePickerComponentText::LaunchPlaceholder),
                "Sélectionner une date de lancement"
            );
        });
    }
}
