use std::time::Duration;

use gpui_kit::component::*;
use gpui_kit::*;

use crate::app::YttmApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YttmTheme {
    pub(crate) transition_generation: usize,
    pub target_mode: Option<ThemeMode>,
}

impl YttmTheme {
    pub(crate) fn new() -> Self {
        Self {
            transition_generation: 0,
            target_mode: None,
        }
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<YttmApp>) {
        let from_mode = self.target_mode.unwrap_or(cx.theme().mode);
        let to_mode = if from_mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };

        self.transition_generation += 1;
        let generation = self.transition_generation;
        self.target_mode = Some(to_mode);

        let from = cx.theme().colors;
        let to = if to_mode.is_dark() {
            *ThemeColor::dark()
        } else {
            *ThemeColor::light()
        };

        cx.spawn(async move |this, cx| {
            let steps = 12;
            for i in 1..=steps {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let t = i as f32 / steps as f32;
                let eased = ease_in_out_cubic(t);
                let colors = lerp_theme_colors(from, to, eased);

                let cont = this.update(cx, |this, _cx| {
                    if this.theme.transition_generation != generation {
                        return false;
                    }
                    Theme::update(_cx, |theme| {
                        theme.colors = colors;
                    });
                    true
                });

                match cont {
                    Ok(true) => {}
                    _ => return,
                }
            }

            let _ = this.update(cx, |this, cx| {
                if this.theme.transition_generation == generation {
                    Theme::change(to_mode, None, cx);
                    this.theme.target_mode = None;
                }
            });
        })
        .detach();
    }
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

fn lerp_theme_colors(from: ThemeColor, to: ThemeColor, t: f32) -> ThemeColor {
    macro_rules! lerp {
        ($c:expr, $from:expr, $to:expr, $t:expr, $($field:ident),+$(,)?) => {
            $(
                $c.$field.h = $from.$field.h + ($to.$field.h - $from.$field.h) * $t;
                $c.$field.s = $from.$field.s + ($to.$field.s - $from.$field.s) * $t;
                $c.$field.l = $from.$field.l + ($to.$field.l - $from.$field.l) * $t;
                $c.$field.a = $from.$field.a + ($to.$field.a - $from.$field.a) * $t;
            )+
        };
    }

    let mut c = from;
    lerp!(
        c,
        from,
        to,
        t,
        background,
        foreground,
        border,
        muted,
        muted_foreground,
        selection,
        accent,
        accent_foreground,
        ring,
        secondary,
        secondary_hover,
        secondary_active,
        secondary_foreground,
        title_bar,
        title_bar_border,
        window_border,
        scrollbar,
        scrollbar_thumb,
        scrollbar_thumb_hover,
        list,
        list_hover,
        list_active,
        list_active_border,
        list_even,
        list_head,
        table,
        table_hover,
        table_active,
        table_active_border,
        table_even,
        table_head,
        table_head_foreground,
        table_row_border,
        overlay,
        input,
        popover,
        popover_foreground,
        tab,
        tab_active,
        tab_active_foreground,
        tab_bar,
        sidebar,
        sidebar_border,
        sidebar_foreground,
        button,
        button_hover,
        button_active,
        button_foreground,
        link,
        caret,
        skeleton,
        group_box,
        drag_border,
        drop_target,
    );
    c
}
