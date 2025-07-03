use gpui::{Action, App, AppContext as _, Entity, Global, SharedString, actions};
use gpui_component::scroll::ScrollbarShow;

use serde::Deserialize;

#[derive(Action, Clone, Deserialize, Eq, PartialEq)]
#[action(namespace = story, no_json)]
pub struct SelectScrollbarShow(pub ScrollbarShow);

#[derive(Action, Clone, Deserialize, Eq, PartialEq)]
#[action(namespace = story, no_json)]
pub struct SelectLocale(pub SharedString);

#[derive(Action, Clone, Deserialize, Eq, PartialEq)]
#[action(namespace = story, no_json)]
pub struct SelectFont(pub usize);

#[derive(Action, Clone, Deserialize, Eq, PartialEq)]
#[action(namespace = story, no_json)]
pub struct SelectRadius(pub usize);

actions!(story, [Quit, Open, CloseWindow, ToggleSearch]);

pub struct AppState {
    pub invisible_panels: Entity<Vec<SharedString>>,
}
impl AppState {
    pub(crate) fn init(cx: &mut App) {
        let state = Self {
            invisible_panels: cx.new(|_| Vec::new()),
        };
        cx.set_global::<AppState>(state);
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}

impl Global for AppState {}
