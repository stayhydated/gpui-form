use es_fluent::{EsFluent, FluentLabel, FluentLocalizer, FluentLocalizerExt as _, FluentMessage};
use es_fluent_manager_embedded as i18n_manager;

es_fluent_manager_embedded::define_i18n_module!();

pub use i18n_manager::{EmbeddedI18n, EmbeddedInitError, LocalizationError};

/// Renders a Fluent message through an explicit caller-owned localizer.
pub fn localize_message<L, T>(localizer: &L, message: &T) -> String
where
    L: FluentLocalizer + ?Sized,
    T: FluentMessage + ?Sized,
{
    localizer.localize_message(message)
}

/// Renders a Fluent type label through an explicit caller-owned localizer.
pub fn localize_label<L, T>(localizer: &L) -> String
where
    L: FluentLocalizer + ?Sized,
    T: FluentLabel,
{
    T::localize_label(localizer)
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "date_picker")]
pub(crate) enum DatePickerText {
    SelectDate,
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "file_picker")]
pub(crate) enum FilePickerText {
    SelectAFile,
    SelectADirectory,
    SelectAFileOrDirectory,
    SelectFile,
    SelectDirectory,
    SelectFileOrDirectory,
    Browse,
    DialogDropped,
    PathsSelected { count: usize },
}

#[cfg(test)]
mod tests {
    use es_fluent::unic_langid::langid;

    use super::*;

    fn strip_fluent_isolates(input: &str) -> String {
        input
            .chars()
            .filter(|ch| !matches!(*ch, '\u{2068}' | '\u{2069}'))
            .collect()
    }

    #[test]
    fn resolves_runtime_component_messages() {
        let i18n = EmbeddedI18n::try_new_with_language(langid!("en")).unwrap();
        assert_eq!(
            i18n.localize_message(&DatePickerText::SelectDate),
            "Select date"
        );
        assert_eq!(i18n.localize_message(&FilePickerText::Browse), "Browse");
        assert_eq!(
            strip_fluent_isolates(
                &i18n.localize_message(&FilePickerText::PathsSelected { count: 2 })
            ),
            "2 paths selected"
        );

        i18n.select_language(langid!("fr-FR")).unwrap();
        assert_eq!(
            i18n.localize_message(&DatePickerText::SelectDate),
            "Sélectionner une date"
        );
        assert_eq!(i18n.localize_message(&FilePickerText::Browse), "Parcourir");
        assert_eq!(
            strip_fluent_isolates(
                &i18n.localize_message(&FilePickerText::PathsSelected { count: 2 })
            ),
            "2 chemins sélectionnés"
        );

        i18n.select_language(langid!("zh-CN")).unwrap();
        assert_eq!(
            i18n.localize_message(&DatePickerText::SelectDate),
            "选择日期"
        );
        assert_eq!(i18n.localize_message(&FilePickerText::Browse), "浏览");
        assert_eq!(
            strip_fluent_isolates(
                &i18n.localize_message(&FilePickerText::PathsSelected { count: 2 })
            ),
            "已选择 2 个路径"
        );
    }
}
