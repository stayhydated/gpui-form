mod font_size_selector;
mod locale_selector;

use self::{font_size_selector::FontSizeSelector, locale_selector::LocaleSelector};
use gpui::{
    AnyElement, App, AppContext as _, ClickEvent, Context, Corner, Entity, Hsla,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, ContextModal as _, IconName, Sizable as _, Theme, ThemeMode, TitleBar,
    badge::Badge,
    button::{Button, ButtonVariants as _},
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    scroll::ScrollbarShow,
};
use std::rc::Rc;

pub struct AppTitleBar {
    title: SharedString,
    locale_selector: Entity<LocaleSelector>,
    font_size_selector: Entity<FontSizeSelector>,
    theme_color: Entity<ColorPickerState>,
    child: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>,
    _subscriptions: Vec<Subscription>,
}

impl AppTitleBar {
    pub fn new(
        title: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let locale_selector = cx.new(|cx| LocaleSelector::new(window, cx));
        let font_size_selector = cx.new(|cx| FontSizeSelector::new(window, cx));

        if cx.should_auto_hide_scrollbars() {
            Theme::global_mut(cx).scrollbar_show = ScrollbarShow::Scrolling;
        } else {
            Theme::global_mut(cx).scrollbar_show = ScrollbarShow::Hover;
        }

        let theme_color =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cx.theme().primary));

        let _subscriptions = vec![cx.subscribe_in(
            &theme_color,
            window,
            |this, _, ev: &ColorPickerEvent, window, cx| match ev {
                ColorPickerEvent::Change(color) => {
                    this.set_theme_color(*color, window, cx);
                },
            },
        )];

        Self {
            title: title.into(),
            locale_selector,
            font_size_selector,
            theme_color,
            child: Rc::new(|_, _| div().into_any_element()),
            _subscriptions,
        }
    }

    pub fn child<F, E>(mut self, f: F) -> Self
    where
        E: IntoElement,
        F: Fn(&mut Window, &mut App) -> E + 'static,
    {
        self.child = Rc::new(move |window, cx| f(window, cx).into_any_element());
        self
    }

    fn set_theme_color(
        &mut self,
        color: Option<Hsla>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(color) = color {
            let theme = cx.global_mut::<Theme>();
            theme.apply_color(color);
            self.theme_color.update(cx, |state, cx| {
                state.set_value(color, window, cx);
            });
            window.refresh();
        }
    }

    fn change_color_mode(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let mode = match cx.theme().mode.is_dark() {
            true => ThemeMode::Light,
            false => ThemeMode::Dark,
        };

        Theme::change(mode, None, cx);
        self.set_theme_color(self.theme_color.read(cx).value(), window, cx);
    }
}

impl Render for AppTitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let notifications_count = window.notifications(cx).len();

        TitleBar::new()
            .child(div().flex().items_center().child(self.title.clone()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .px_2()
                    .gap_2()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child((self.child.clone())(window, cx))
                    .child(
                        ColorPicker::new(&self.theme_color)
                            .small()
                            .anchor(Corner::TopRight)
                            .icon(IconName::Palette),
                    )
                    .child(
                        Button::new("theme-mode")
                            .map(|this| {
                                if cx.theme().mode.is_dark() {
                                    this.icon(IconName::Sun)
                                } else {
                                    this.icon(IconName::Moon)
                                }
                            })
                            .small()
                            .ghost()
                            .on_click(cx.listener(Self::change_color_mode)),
                    )
                    .child(self.locale_selector.clone())
                    .child(self.font_size_selector.clone())
                    .child(
                        Button::new("github")
                            .icon(IconName::GitHub)
                            .small()
                            .ghost()
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/longbridge/gpui-component")
                            }),
                    )
                    .child(
                        div().relative().child(
                            Badge::new().count(notifications_count).max(99).child(
                                Button::new("bell")
                                    .small()
                                    .ghost()
                                    .compact()
                                    .icon(IconName::Bell),
                            ),
                        ),
                    ),
            )
    }
}
