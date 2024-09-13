use makepad_widgets::Cx;

pub mod actions;
pub mod computed_list;
pub mod desktop_buttons;
pub mod download_notification_popup;
pub mod external_link;
pub mod modal;
pub mod popup_notification;
pub mod resource_imports;
pub mod styles;
pub mod tooltip;
pub mod utils;
pub mod widgets;

pub fn live_design(cx: &mut Cx) {
    computed_list::live_design(cx);
    styles::live_design(cx);
    resource_imports::live_design(cx);
    widgets::live_design(cx);
    modal::live_design(cx);
    popup_notification::live_design(cx);
    external_link::live_design(cx);
    download_notification_popup::live_design(cx);
    tooltip::live_design(cx);
    desktop_buttons::live_design(cx);
}
