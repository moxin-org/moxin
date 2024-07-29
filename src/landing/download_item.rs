use crate::shared::{
    actions::DownloadAction,
    utils::{format_model_downloaded_size, format_model_size},
};
use makepad_widgets::*;
use moxin_protocol::data::{FileID, PendingDownload, PendingDownloadsStatus};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::MoxinButton;

    ICON_PAUSE = dep("crate://self/resources/icons/pause_download.svg")
    ICON_CANCEL = dep("crate://self/resources/icons/cancel_download.svg")
    ICON_PLAY = dep("crate://self/resources/icons/play_download.svg")
    ICON_RETRY = dep("crate://self/resources/icons/retry_download.svg")

    ModelAttributeTag = <RoundedView> {
        width: Fit,
        height: Fit,
        padding: {top: 6, bottom: 6, left: 10, right: 10}

        spacing: 5,
        draw_bg: {
            radius: 2.0,
        }

        caption = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                color: #fff
            }
        }
    }

    Information = <View> {
        width: 380,
        height: Fit,
        flow: Right,
        spacing: 12,
        margin: {right: 60}

        align: {x: 0.0, y: 0.5},

        architecture_tag = <ModelAttributeTag> {
            caption = {
                text: "StableLM"
            }
            draw_bg: {
                color: #A44EBB,
            }
        }

        params_size_tag = <ModelAttributeTag> {
            caption = {
                text: "3B"
            }
            draw_bg: {
                color: #44899A,
            }
        }

        filename = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 10},
                color: #000
            }
            text: "Stable-code-instruct-3b-Q8_0.gguf"
        }
    }

    Progress = <View> {
        width: 600,
        height: Fit,
        spacing: 8,

        flow: Down,

        <View> {
            width: Fill,
            height: Fit,

            flow: Right,

            progress = <Label> {
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 9},
                    color: #099250
                }
                text: "Downloading 9.7%"
            }
            <View> { width: Fill, height: 1 }
            downloaded_size = <Label> {
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 9},
                    color: #667085
                }
                text: "288.55 MB / 2.97 GB | 10.59 MB/s "
            }
        }

        <View> {
            width: Fill,
            height: 12,

            flow: Overlay,

            <RoundedView> {
                width: 600,
                height: Fill,
                draw_bg: {
                    color: #D9D9D9,
                    radius: 2.0,
                }
            }

            progress_bar = <RoundedView> {
                width: 0,
                height: Fill,
                draw_bg: {
                    color: #099250,
                    radius: 2.0,
                }
            }
        }
    }

    ActionButton = <MoxinButton> {
        width: 40,
        height: 40,

        draw_bg: {
            border_color: #EAECF0,
            border_width: 1.0,
            color: #fff,
            color_hover: #E2F1F1,
            radius: 2.0,
        }

        draw_icon: {
            color: #667085;
        }
    }

    Actions = <View> {
        width: Fill,
        height: 40,
        flow: Right,
        spacing: 12,

        align: {x: 0.5, y: 0.5},

        pause_button = <ActionButton> {
            draw_icon: {
                svg_file: (ICON_PAUSE),
            }
            icon_walk: { margin: { left: 6 } }
        }

        play_button = <ActionButton> {
            draw_icon: {
                svg_file: (ICON_PLAY),
            }
            icon_walk: { margin: { left: 6 } }
        }

        retry_button = <ActionButton> {
            draw_icon: {
                svg_file: (ICON_RETRY),
            }
        }

        cancel_button = <ActionButton> {
            draw_icon: {
                svg_file: (ICON_CANCEL),
            }
            icon_walk: { margin: 0 }
        }
    }

    DownloadItem = {{DownloadItem}}<RoundedView> {
        width: Fill,
        height: Fit,

        padding: 20,
        margin: {bottom: 16},
        spacing: 30,
        align: {x: 0.0, y: 0.5},

        draw_bg: {
            border_color: #EAECF0,
            border_width: 1.0,
            color: #fff,
        }

        <Information> {}
        <Progress> {}
        <Actions> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct DownloadItem {
    #[deref]
    view: View,

    #[rust]
    file_id: Option<FileID>,
}

impl Widget for DownloadItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let download = scope.data.get::<PendingDownload>().unwrap();
        self.file_id = Some(download.file.id.clone());

        self.label(id!(filename))
            .set_text(download.file.name.as_str());

        self.label(id!(architecture_tag.caption))
            .set_text(download.model.architecture.as_str());

        self.label(id!(params_size_tag.caption))
            .set_text(&&download.model.requires.as_str());

        let progress_bar_width = download.progress * 6.0; // 6.0 = 600px / 100%
        let label = self.label(id!(progress));
        match download.status {
            PendingDownloadsStatus::Initializing => {
                let downloading_color = vec3(0.035, 0.572, 0.314); //#099250

                label.set_text(&format!("Downloading {:.1}%", download.progress));
                label.apply_over(
                    cx,
                    live! { draw_text: { color: (downloading_color) }
                    },
                );

                self.view(id!(progress_bar)).apply_over(
                    cx,
                    live! {
                        width: (progress_bar_width)
                        draw_bg: { color: (downloading_color) }
                    },
                );

                self.button(id!(pause_button)).set_visible(false);
                self.button(id!(play_button)).set_visible(false);
                self.button(id!(retry_button)).set_visible(false);
                self.button(id!(cancel_button)).set_visible(false);
            }
            PendingDownloadsStatus::Downloading => {
                let downloading_color = vec3(0.035, 0.572, 0.314); //#099250

                label.set_text(&format!("Downloading {:.1}%", download.progress));
                label.apply_over(
                    cx,
                    live! { draw_text: { color: (downloading_color) }
                    },
                );

                self.view(id!(progress_bar)).apply_over(
                    cx,
                    live! {
                        width: (progress_bar_width)
                        draw_bg: { color: (downloading_color) }
                    },
                );

                self.button(id!(pause_button)).set_visible(true);
                self.button(id!(play_button)).set_visible(false);
                self.button(id!(retry_button)).set_visible(false);
                self.button(id!(cancel_button)).set_visible(true);
            }
            PendingDownloadsStatus::Paused => {
                let paused_color = vec3(0.4, 0.44, 0.52); //#667085

                label.set_text(&format!("Paused {:.1}%", download.progress));
                label.apply_over(
                    cx,
                    live! { draw_text: { color: (paused_color) }
                    },
                );

                self.view(id!(progress_bar)).apply_over(
                    cx,
                    live! {
                        width: (progress_bar_width)
                        draw_bg: { color: (paused_color) }
                    },
                );

                self.button(id!(pause_button)).set_visible(false);
                self.button(id!(play_button)).set_visible(true);
                self.button(id!(retry_button)).set_visible(false);
                self.button(id!(cancel_button)).set_visible(true);
            }
            PendingDownloadsStatus::Error => {
                let failed_color = vec3(0.7, 0.11, 0.09); // #B42318

                label.set_text(&format!("Error {:.1}%", download.progress));
                label.apply_over(
                    cx,
                    live! { draw_text: { color: (failed_color) }
                    },
                );

                self.view(id!(progress_bar)).apply_over(
                    cx,
                    live! {
                        width: (progress_bar_width)
                        draw_bg: { color: (failed_color) }
                    },
                );

                self.button(id!(pause_button)).set_visible(false);
                self.button(id!(play_button)).set_visible(false);
                self.button(id!(retry_button)).set_visible(true);
                self.button(id!(cancel_button)).set_visible(true);
            }
        }

        let total_size = format_model_size(&download.file.size).unwrap_or("-".to_string());
        let downloaded_size = format_model_downloaded_size(&download.file.size, download.progress)
            .unwrap_or("-".to_string());

        self.label(id!(downloaded_size))
            .set_text(&format!("{} / {}", downloaded_size, total_size));

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for DownloadItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        for button_id in [id!(play_button), id!(retry_button)] {
            if self.button(button_id).clicked(&actions) {
                let Some(file_id) = &self.file_id else { return };
                let widget_uid = self.widget_uid();
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    DownloadAction::Play(file_id.clone()),
                );
                self.button(button_id).reset_hover(cx)
            }
        }

        if self.button(id!(pause_button)).clicked(&actions) {
            let Some(file_id) = &self.file_id else { return };
            let widget_uid = self.widget_uid();
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Pause(file_id.clone()),
            );
            self.button(id!(pause_button)).reset_hover(cx)
        }

        if self.button(id!(cancel_button)).clicked(&actions) {
            let Some(file_id) = &self.file_id else { return };
            let widget_uid = self.widget_uid();
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Cancel(file_id.clone()),
            );
            self.button(id!(cancel_button)).reset_hover(cx)
        }
    }
}
