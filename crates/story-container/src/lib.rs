use story_container_core;

#[cfg(feature = "macros")]
pub use story_container_macros::*;

pub use assets::*;
use gpui::prelude::{
    Context, FluentBuilder as _, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _,
};
use gpui::{App, AppContext as _, ClickEvent, Entity, Subscription, Window, div, px, relative};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, h_flex,
    input::{InputEvent, InputState, TextInput},
    resizable::{ResizableState, h_resizable, resizable_panel},
    sidebar::{Sidebar, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex,
};
use story::StoryContainer;

pub struct Gallery {
    stories: Vec<Entity<StoryContainer>>,
    active_index: Option<usize>,
    collapsed: bool,
    search_input: Entity<InputState>,
    sidebar_state: Entity<ResizableState>,
    _subscriptions: Vec<Subscription>,
}

impl Gallery {
    pub fn new(
        initial_stories: Vec<Entity<StoryContainer>>,
        init_story_name: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search_input =
            cx.new(|cx_input| InputState::new(window, cx_input).placeholder("Search..."));

        let subscriptions = vec![
            #[allow(clippy::single_match)]
            cx.subscribe(&search_input, |this, _, event, cx_window| match event {
                InputEvent::Change(_) => {
                    let query = this
                        .search_input
                        .read(cx_window)
                        .value()
                        .trim()
                        .to_lowercase();
                    let filtered_stories_on_change: Vec<_> = this
                        .stories
                        .iter()
                        .filter(|story| {
                            let title = if let Some(title_fn) = &story.read(cx_window).title_fn {
                                title_fn()
                            } else {
                                story.read(cx_window).name.to_string()
                            };
                            title.to_lowercase().contains(&query)
                        })
                        .cloned()
                        .collect();

                    if let Some(first_filtered_story) = filtered_stories_on_change.first()
                        && let Some(original_idx) =
                            this.stories.iter().position(|s| s == first_filtered_story)
                    {
                        this.active_index = Some(original_idx);
                    } else {
                        this.active_index = None;
                    }
                    cx_window.notify();
                },
                _ => {},
            }),
        ];

        let mut this = Self {
            search_input,
            stories: initial_stories.clone(),
            active_index: if initial_stories.is_empty() {
                None
            } else {
                Some(0)
            },
            collapsed: false,
            sidebar_state: ResizableState::new(cx),
            _subscriptions: subscriptions,
        };

        if let Some(name) = init_story_name {
            this.set_active_story(name, cx);
        }

        this
    }

    fn set_active_story(&mut self, name: &str, app_cx: &App) {
        let lowercase_name = name.to_lowercase().replace("story", "");
        let story_index = self.stories.iter().position(|story_entity| {
            let story_data = story_entity.read(app_cx);
            let title = if let Some(title_fn) = &story_data.title_fn {
                title_fn()
            } else {
                story_data.name.to_string()
            };
            title.to_lowercase().replace("story", "") == lowercase_name
        });

        if let Some(index) = story_index {
            self.active_index = Some(index);
        }
    }

    pub fn view(
        initial_stories: Vec<Entity<StoryContainer>>,
        init_story_name: Option<&str>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx_self| Self::new(initial_stories, init_story_name, window, cx_self))
    }
}

impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.search_input.read(cx).value().trim().to_lowercase();

        let filtered_stories: Vec<Entity<StoryContainer>> = self
            .stories
            .iter()
            .filter(|story| {
                let title = if let Some(title_fn) = &story.read(cx).title_fn {
                    title_fn()
                } else {
                    story.read(cx).name.to_string()
                };
                title.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();

        let mut active_story_to_render: Option<Entity<StoryContainer>> = None;
        let mut ui_active_index_in_filtered_list: Option<usize> = None;

        if let Some(current_original_idx) = self.active_index
            && let Some(story_from_original_list) = self.stories.get(current_original_idx)
            && let Some(idx_in_filtered) = filtered_stories
                .iter()
                .position(|s| s == story_from_original_list)
        {
            active_story_to_render = Some(story_from_original_list.clone());
            ui_active_index_in_filtered_list = Some(idx_in_filtered);
        }

        let (story_name, description) =
            if let Some(story_to_render_cloned) = active_story_to_render.as_ref() {
                let story_data = story_to_render_cloned.read(cx);
                let title = if let Some(title_fn) = &story_data.title_fn {
                    title_fn()
                } else {
                    story_data.name.to_string()
                };
                let desc = if let Some(desc_fn) = &story_data.description_fn {
                    desc_fn()
                } else {
                    story_data.description.to_string()
                };
                (title, desc)
            } else {
                ("".to_owned(), "".to_owned())
            };

        h_resizable("gallery-container", self.sidebar_state.clone())
            .child(
                resizable_panel()
                    .size(px(255.))
                    .size_range(px(200.)..px(320.))
                    .child(
                        Sidebar::left()
                            .width(relative(1.))
                            .border_width(px(0.))
                            .collapsed(self.collapsed)
                            .header(
                                v_flex()
                                    .w_full()
                                    .gap_4()
                                    .child(
                                        SidebarHeader::new()
                                            .w_full()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .rounded(cx.theme().radius)
                                                    .bg(cx.theme().primary)
                                                    .text_color(cx.theme().primary_foreground)
                                                    .size_8()
                                                    .flex_shrink_0()
                                                    .when(!self.collapsed, |this| {
                                                        this.child(Icon::new(
                                                            IconName::GalleryVerticalEnd,
                                                        ))
                                                    })
                                                    .when(self.collapsed, |this| {
                                                        this.size_4()
                                                            .bg(cx.theme().transparent)
                                                            .text_color(cx.theme().foreground)
                                                            .child(Icon::new(
                                                                IconName::GalleryVerticalEnd,
                                                            ))
                                                    })
                                                    .rounded_lg(),
                                            )
                                            .when(!self.collapsed, |this| {
                                                this.child(
                                                    v_flex()
                                                        .gap_0()
                                                        .text_sm()
                                                        .flex_1()
                                                        .line_height(relative(1.25))
                                                        .overflow_hidden()
                                                        .text_ellipsis()
                                                        .child("GPUI Component")
                                                        .child(
                                                            div()
                                                                .text_color(
                                                                    cx.theme().muted_foreground,
                                                                )
                                                                .child("Gallery")
                                                                .text_xs(),
                                                        ),
                                                )
                                            }),
                                    )
                                    .child(
                                        div()
                                            .bg(cx.theme().sidebar_border)
                                            .px_1()
                                            .rounded_full()
                                            .flex_1()
                                            .mx_1()
                                            .child(
                                                TextInput::new(&self.search_input)
                                                    .appearance(false)
                                                    .cleanable(),
                                            ),
                                    ),
                            )
                            .child(SidebarMenu::new().children(
                                filtered_stories.iter().enumerate().map(
                                    |(idx_in_filtered, story_entity_in_filtered)| {
                                        let story_data = story_entity_in_filtered.read(cx);
                                        let name = if let Some(title_fn) = &story_data.title_fn {
                                            title_fn().into()
                                        } else {
                                            story_data.name.clone()
                                        };
                                        let is_active = ui_active_index_in_filtered_list
                                            == Some(idx_in_filtered);

                                        let story_entity_for_click =
                                            story_entity_in_filtered.clone();

                                        SidebarMenuItem::new(name).active(is_active).on_click(
                                            cx.listener(
                                                move |this, _: &ClickEvent, _, cx_listener| {
                                                    if let Some(original_idx) = this
                                                        .stories
                                                        .iter()
                                                        .position(|s| s == &story_entity_for_click)
                                                    {
                                                        this.active_index = Some(original_idx);
                                                    }
                                                    cx_listener.notify();
                                                },
                                            ),
                                        )
                                    },
                                ),
                            )),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .overflow_x_hidden()
                    .child(
                        h_flex()
                            .id("header")
                            .p_4()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .justify_between()
                            .items_start()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(div().text_xl().child(story_name))
                                    .child(
                                        div()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(description),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("story")
                            .flex_1()
                            .overflow_y_scroll()
                            .when_some(active_story_to_render, |this, active_story_ref| {
                                this.child(active_story_ref.clone())
                            }),
                    )
                    .into_any_element(),
            )
    }
}
