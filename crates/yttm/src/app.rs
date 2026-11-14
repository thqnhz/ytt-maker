use gpui_kit::base::ResizableState;
use gpui_kit::*;

use yttm_core::{Cue, Document};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceLayout {
    Stacked,
    SideBySide,
}

pub struct YttmApp {
    pub document: Document,
    pub selected_cue: Option<usize>,
    pub workspace_layout: WorkspaceLayout,
    pub stacked_split: Entity<ResizableState>,
    pub side_split: Entity<ResizableState>,
}

impl YttmApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            document: Document {
                cues: sample_cues(),
            },
            selected_cue: None,
            workspace_layout: WorkspaceLayout::Stacked,
            stacked_split: cx.new(|_| ResizableState::default()),
            side_split: cx.new(|_| ResizableState::default()),
        }
    }

    /// Append a cue entry at the end
    pub fn append_cue(&mut self) {
        let start_ms = self.document.cues.last().map(|cue| cue.end_ms).unwrap_or(0);
        let index = self.document.cues.len();
        self.add_cue_at(index, start_ms, start_ms + 2_000);
    }

    pub fn add_cue_above_selected(&mut self) {
        let Some(ix) = self.selected_cue else {
            return;
        };

        let end_ms = self.document.cues[ix].start_ms;
        let start_ms = end_ms.saturating_sub(2_000);
        self.add_cue_at(ix, start_ms, end_ms);
    }

    pub fn add_cue_below_selected(&mut self) {
        let Some(ix) = self.selected_cue else {
            return;
        };

        let start_ms = self.document.cues[ix].end_ms;
        self.add_cue_at(ix + 1, start_ms, start_ms + 2_000);
    }

    pub fn delete_selected_cue(&mut self) {
        let Some(ix) = self.selected_cue else {
            return;
        };

        self.document.cues.remove(ix);

        self.selected_cue = if self.document.cues.is_empty() {
            // Empty -> Unselect
            None
        } else if ix == 0 {
            // First entry -> keep first entry
            Some(0)
        } else {
            // Non-first entry -> to above entry
            Some(ix - 1)
        }
    }

    pub fn run() {
        gpui_kit::application()
            .with_assets(gpui_kit::assets::AllAssets)
            .run(|cx| {
                gpui_kit::init(cx);
                gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                    cx.new(|cx| YttmApp::new(window, cx))
                })
                .expect("failed to open window");
            });
    }

    fn add_cue_at(&mut self, index: usize, start_ms: u64, end_ms: u64) {
        self.document.cues.insert(
            index,
            Cue {
                start_ms,
                end_ms,
                text: String::new(),
                pos_x: None,
                pos_y: None,
            },
        );
        self.selected_cue = Some(index);
    }
}

fn sample_cues() -> Vec<Cue> {
    vec![
        Cue {
            start_ms: 1_000,
            end_ms: 3_500,
            text: "First line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            start_ms: 4_000,
            end_ms: 6_200,
            text: "Second line, positioned".into(),
            pos_x: Some(50),
            pos_y: Some(80),
        },
        Cue {
            start_ms: 7_000,
            end_ms: 9_000,
            text: "Third line".into(),
            pos_x: Some(10),
            pos_y: None,
        },
        Cue {
            start_ms: 9_000,
            end_ms: 11_000,
            text: "Fourth line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            start_ms: 15_000,
            end_ms: 20_000,
            text: "Fifth line".into(),
            pos_x: None,
            pos_y: None,
        },
        Cue {
            start_ms: 21_000,
            end_ms: 23_000,
            text: "Sixth line".into(),
            pos_x: None,
            pos_y: None,
        },
    ]
}
