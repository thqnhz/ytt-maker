use gpui_kit::assets::IconName;
use gpui_kit::base::ResizableState;
use gpui_kit::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceLayout {
    Stacked,
    SideBySide,
}

pub(crate) struct WorkspaceSplit {
    pub stacked: Entity<ResizableState>,
    pub side_by_side: Entity<ResizableState>,
}

pub(crate) struct Workspace {
    pub layout: WorkspaceLayout,
    pub split: WorkspaceSplit,
}

impl Workspace {
    pub(crate) fn layout_tooltip(l: &WorkspaceLayout) -> String {
        match l {
            WorkspaceLayout::Stacked => "Switch to Side by side".to_owned(),
            WorkspaceLayout::SideBySide => "Switch to Stacked".to_owned(),
        }
    }

    pub(crate) fn layout_icon(l: &WorkspaceLayout) -> IconName {
        match l {
            WorkspaceLayout::Stacked => IconName::PanelBottom,
            WorkspaceLayout::SideBySide => IconName::PanelRight,
        }
    }

    pub(crate) fn switch_layout(&mut self) {
        self.layout = match self.layout {
            WorkspaceLayout::SideBySide => WorkspaceLayout::Stacked,
            WorkspaceLayout::Stacked => WorkspaceLayout::SideBySide,
        };
    }
}
