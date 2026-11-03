use es_fluent::EsFluent;
use es_fluent_manager_embedded as i18n_manager;

es_fluent_manager_embedded::define_i18n_module!();

pub use i18n_manager::{EmbeddedI18n, EmbeddedInitError, LocalizationError};

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
