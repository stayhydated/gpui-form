use crate::story::StoryContainer;

/// Entry type for story registration
pub struct StoryEntry {
    pub name: &'static str,
    pub create_fn: fn(&mut ::gpui::Window, &mut ::gpui::App) -> ::gpui::Entity<StoryContainer>,
}

inventory::collect!(StoryEntry);

/// Entry type for init function registration
pub struct InitEntry {
    pub init_fn: fn(&mut ::gpui::App),
}

inventory::collect!(InitEntry);
