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
    pub selected_cue: Option<u64>,
    next_cue_id: u64,
    pub workspace_layout: WorkspaceLayout,
    pub stacked_split: Entity<ResizableState>,
    pub side_split: Entity<ResizableState>,
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
        }
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
