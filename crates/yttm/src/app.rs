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
            workspace_layout: WorkspaceLayout::Stacked,
            stacked_split: cx.new(|_| ResizableState::default()),
            side_split: cx.new(|_| ResizableState::default()),
        }
    }

    pub fn add_cue(&mut self) {
        let start_ms = self.document.cues.last().map(|cue| cue.end_ms).unwrap_or(0);

        self.document.cues.push(Cue {
            start_ms,
            end_ms: start_ms + 2_000,
            text: String::new(),
            pos_x: None,
            pos_y: None,
        });
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
