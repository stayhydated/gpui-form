use crate::story::StoryContainer;
use proc_macro2::TokenStream;
use quote::quote;

/// Entry type for story registration
pub struct StoryEntry {
    pub name: &'static str,
    pub create_fn: fn(&mut ::gpui::Window, &mut ::gpui::App) -> ::gpui::Entity<StoryContainer>,
}

/// Entry type for init function registration
pub struct InitEntry {
    pub init_fn: fn(&mut ::gpui::App),
}

pub fn generate_init() -> TokenStream {
    quote! {
        pub fn init(cx: &mut ::gpui::App) {
            for entry in inventory::iter::<InitEntry> {
                (entry.init_fn)(cx);
            }
        }
    }
}

/// Generate the function that creates all registered stories
pub fn generate_stories() -> TokenStream {
    quote! {
        pub fn generate_stories(window: &mut ::gpui::Window, cx: &mut ::gpui::App) -> Vec<::gpui::Entity<::story_container::story::StoryContainer>> {
            let mut stories = Vec::new();
            for entry in inventory::iter::<StoryEntry> {
                stories.push((entry.create_fn)(window, cx));
            }
            stories
        }
    }
}
