use gpui_kit::component::input::*;
use gpui_kit::*;

pub struct YttmInput {
    pub subtitle: Entity<InputState>,
    pub start: Entity<InputState>,
    pub end: Entity<InputState>,
}
