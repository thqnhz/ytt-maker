use std::borrow::Cow;
use std::time::Duration;

use gpui_kit::base::ResizableState;
use gpui_kit::component::*;
use gpui_kit::*;

use yttm_core::{Cue, Document};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceLayout {
    Stacked,
    SideBySide,
}

pub struct YttmApp {
    pub document: Document,
    pub selected_cue: Option<u64>,
    next_cue_id: u64,
    pub workspace_layout: WorkspaceLayout,
    pub stacked_split: Entity<ResizableState>,
    pub side_split: Entity<ResizableState>,
    theme_transition_generation: usize,
    pub theme_target_mode: Option<ThemeMode>,
}

impl YttmApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut next_cue_id = 1;
        let cues = sample_cues(&mut next_cue_id);
        Self {
            document: Document { cues },
            selected_cue: None,
            next_cue_id,
            workspace_layout: WorkspaceLayout::Stacked,
            stacked_split: cx.new(|_| ResizableState::default()),
            side_split: cx.new(|_| ResizableState::default()),
            theme_transition_generation: 0,
            theme_target_mode: None,
        }
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        let from_mode = self.theme_target_mode.unwrap_or(cx.theme().mode);
        let to_mode = if from_mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };

        self.theme_transition_generation += 1;
        let generation = self.theme_transition_generation;
        self.theme_target_mode = Some(to_mode);

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
                    if this.theme_transition_generation != generation {
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
                if this.theme_transition_generation == generation {
                    Theme::change(to_mode, None, cx);
                    this.theme_target_mode = None;
                }
            });
        })
        .detach();
    }

    /// Append a cue entry at the end
    pub fn append_cue(&mut self) {
        let start_ms = self.document.cues.last().map(|cue| cue.end_ms).unwrap_or(0);
        self.insert_cue(start_ms, start_ms + 2_000);
    }

    pub fn add_cue_above_selected(&mut self) {
        let Some(selected) = self.selected_cue() else {
            return;
        };

        let end_ms = selected.start_ms;
        let start_ms = end_ms.saturating_sub(2_000);
        self.insert_cue(start_ms, end_ms);
    }

    pub fn add_cue_below_selected(&mut self) {
        let Some(selected) = self.selected_cue() else {
            return;
        };

        let start_ms = selected.end_ms;
        self.insert_cue(start_ms, start_ms + 2_000);
    }

    pub fn delete_selected_cue(&mut self) {
        let Some(selected_id) = self.selected_cue else {
            return;
        };
        let Some(ix) = self.selected_index() else {
            self.selected_cue = None;
            return;
        };

        self.document.cues.remove(ix);

        self.selected_cue = if self.document.cues.is_empty() {
            // Empty -> Unselect
            None
        } else if ix == 0 {
            // First entry -> keep first entry
            Some(self.document.cues[0].id)
        } else {
            // Non-first entry -> to above entry
            Some(self.document.cues[ix - 1].id)
        };
        let _ = selected_id;
    }

    pub fn run() {
        gpui_kit::application()
            .with_assets(gpui_kit::assets::AllAssets)
            .run(|cx| {
                gpui_kit::init(cx);

                cx.text_system()
                    .add_fonts(vec![
                        Cow::Borrowed(include_bytes!(
                            "../../../assets/fonts/Inter/Inter-VariableFont_opsz,wght.ttf"
                        )),
                        Cow::Borrowed(include_bytes!(
                            "../../../assets/fonts/Inter/Inter-Italic-VariableFont_opsz,wght.ttf"
                        )),
                    ])
                    .expect("failed to load bundled UI font");

                Theme::update(cx, |theme| {
                    theme.font_family = "Inter".into();
                });

                gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                    cx.new(|cx| YttmApp::new(window, cx))
                })
                .expect("failed to open window");
            });
    }

    fn insert_cue(&mut self, start_ms: u64, end_ms: u64) {
        let id = self.next_cue_id;
        self.next_cue_id += 1;

        self.document.cues.push(Cue {
            id,
            start_ms,
            end_ms,
            text: String::new(),
            pos_x: None,
            pos_y: None,
        });
        self.sort_cues();
        self.selected_cue = Some(id);
    }

    fn sort_cues(&mut self) {
        self.document.cues.sort_by_key(|cue| (cue.start_ms, cue.id));
    }

    fn selected_cue(&self) -> Option<&Cue> {
        let id = self.selected_cue?;
        self.document.cues.iter().find(|cue| cue.id == id)
    }

    fn selected_index(&self) -> Option<usize> {
        let id = self.selected_cue?;
        self.document.cues.iter().position(|cue| cue.id == id)
    }
}

fn sample_cues(next_cue_id: &mut u64) -> Vec<Cue> {
    let mut cues = vec![
        Cue {
            id: 0,
            start_ms: 1_000,
            end_ms: 3_500,
            text: "First line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 4_000,
            end_ms: 6_200,
            text: "Second line, positioned".into(),
            pos_x: Some(50),
            pos_y: Some(80),
        },
        Cue {
            id: 0,
            start_ms: 7_000,
            end_ms: 9_000,
            text: "Third line".into(),
            pos_x: Some(10),
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 9_000,
            end_ms: 11_000,
            text: "Fourth line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 15_000,
            end_ms: 20_000,
            text: "Fifth line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            id: 0,
            start_ms: 21_000,
            end_ms: 23_000,
            text: "Sixth line".into(),
            pos_x: None,
            pos_y: None,
        },
    ];

    for cue in &mut cues {
        cue.id = *next_cue_id;
        *next_cue_id += 1;
    }

    cues
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
