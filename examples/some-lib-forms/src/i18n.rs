use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use strum::EnumIter;

es_fluent_manager_embedded::define_i18n_module!();

pub use gpui_es_fluent::{
    EmbeddedI18n, EmbeddedInitError, I18n, LocalizationError, change_locale, fallback_label,
    localize_label, localize_message, replace_with_language as init,
};

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, EsFluent, PartialEq)]
pub enum Languages {}

#[cfg(test)]
mod tests {
    use es_fluent::FluentLabel as _;
    use some_lib::structs::{empty::Empty, user::User};

    #[test]
    fn resolves_form_labels() {
        let i18n = es_fluent_manager_embedded::EmbeddedI18n::try_new_with_language(
            es_fluent::unic_langid::langid!("en"),
        )
        .unwrap();
        assert_eq!(User::localize_label(&i18n), "User");
        assert_eq!(Empty::localize_label(&i18n), "Empty");

        i18n.select_language(es_fluent::unic_langid::langid!("fr-FR"))
            .unwrap();
        assert_eq!(User::localize_label(&i18n), "Utilisateur");
        assert_eq!(Empty::localize_label(&i18n), "Vide");

        i18n.select_language(es_fluent::unic_langid::langid!("zh-CN"))
            .unwrap();
        assert_eq!(User::localize_label(&i18n), "用户");
        assert_eq!(Empty::localize_label(&i18n), "空");
    }
}
