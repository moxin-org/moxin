use makepad_widgets::Cx;

pub mod actions;
pub mod download_notification_popup;
pub mod external_link;
pub mod portal;
pub mod modal;
pub mod resource_imports;
pub mod styles;
pub mod tooltip;
pub mod utils;
pub mod widgets;

pub fn live_design(cx: &mut Cx) {
    styles::live_design(cx);
    resource_imports::live_design(cx);
    widgets::live_design(cx);
    portal::live_design(cx);
    modal::live_design(cx);
    external_link::live_design(cx);
    download_notification_popup::live_design(cx);
    tooltip::live_design(cx);
}
