#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cue {
    pub id: u64,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    pub cues: Vec<Cue>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceLayout {
    Stacked,
    SideBySide,
}
