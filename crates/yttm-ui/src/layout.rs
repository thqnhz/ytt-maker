use gpui_kit::assets::IconName;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceLayout {
    Stacked,
    SideBySide,
}

pub(crate) fn layout_tooltip(l: WorkspaceLayout) -> String {
    match l {
        WorkspaceLayout::Stacked => "Switch to Side by side".to_owned(),
        WorkspaceLayout::SideBySide => "Switch to Stacked".to_owned(),
    }
}

pub(crate) fn layout_icon(l: WorkspaceLayout) -> IconName {
    match l {
        WorkspaceLayout::Stacked => IconName::PanelBottom,
        WorkspaceLayout::SideBySide => IconName::PanelRight,
    }
}
