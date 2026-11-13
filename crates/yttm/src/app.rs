use gpui_kit::base::ResizableState;
use gpui_kit::*;

use yttm_core::{Cue, Document};

pub struct YttmApp {
    pub document: Document,
    pub main_split: Entity<ResizableState>,
}

impl YttmApp {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            document: Document {
                cues: sample_cues(),
            },
            main_split: cx.new(|_| ResizableState::default()),
        }
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
    ]
}
