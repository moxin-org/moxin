use makepad_widgets::*;
use moxin_protocol::data::{File, FileID, PendingDownloadsStatus};

use super::model_files_tags::ModelFilesTagsWidgetExt;
use crate::{
    data::store::FileWithDownloadInfo,
    shared::{
        actions::{ChatAction, ChatHandler, DownloadAction},
        utils::format_model_size,
    },
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::MoxinButton;
    import crate::landing::model_files_tags::ModelFilesTags;

    ICON_DOWNLOAD = dep("crate://self/resources/icons/download.svg")
    START_CHAT = dep("crate://self/resources/icons/start_chat.svg")

    ICON_PAUSE = dep("crate://self/resources/icons/pause_download.svg")
    ICON_CANCEL = dep("crate://self/resources/icons/cancel_download.svg")
    ICON_PLAY = dep("crate://self/resources/icons/play_download.svg")
    ICON_RETRY = dep("crate://self/resources/icons/retry_download.svg")

    ModelFilesRow = <RoundedYView> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #00f
            radius: vec2(1.0, 1.0)
        }

        cell1 = <View> { width: Fill, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell2 = <View> { width: 140, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell3 = <View> { width: 340, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
        cell4 = <View> { width: 250, height: 56, padding: 10, align: {x: 0.0, y: 0.5} }
    }

    ModelCardButton = <MoxinButton> {
        width: 140,
        height: 32,
    }

    DownloadButton = <ModelCardButton> {
        draw_bg: { color: #099250, border_color: #099250 }
        text: "Download"
        draw_icon: {
            svg_file: (ICON_DOWNLOAD),
        }
    }

    StartChatButton = <ModelCardButton> {
        draw_bg: { color: #fff, color_hover: #09925033, border_color: #d0d5dd }
        text: "Chat with Model"
        draw_text: {
            color: #087443;
        }
        draw_icon: {
            svg_file: (START_CHAT),
            color: #087443
        }
    }

    DownloadPendingButton = <MoxinButton> {
        width: 25,
        height: 25,
        padding: 4,
        draw_icon: {
            fn get_color(self) -> vec4 {
                return #667085;
            }
        }
    }

    DownloadPendingControls = <View> {
        align: {y: 0.5},
        spacing: 8,
        progress_bar = <View> {
            width: 74,
            height: 12,
            flow: Overlay,

            <RoundedView> {
                height: Fill,
                draw_bg: {
                    color: #D9D9D9,
                    radius: 2.5,
                }
            }

            progress_fill = <RoundedView> {
                width: 0,
                height: Fill,
                draw_bg: {
                    radius: 2.5,
                }
            }
        }
        progress_text_layout = <View> {
            width: 40,
            align: {x: 1, y: 0.5},
            progress_text = <Label> {
                text: "0%",
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 9},
                }
            }
        }

        resume_download_button = <DownloadPendingButton> {
            icon_walk: { margin: { left: 4 } }
            draw_icon: {
                svg_file: (ICON_PLAY),
            }
        }
        retry_download_button = <DownloadPendingButton> {
            draw_icon: {
                svg_file: (ICON_RETRY),
            }
        }
        pause_download_button = <DownloadPendingButton> {
            icon_walk: { margin: { left: 4 } }
            draw_icon: {
                svg_file: (ICON_PAUSE),
            }
        }
        cancel_download_button = <DownloadPendingButton> {
            draw_icon: {
                svg_file: (ICON_CANCEL),
            }
        }
    }

    ModelFilesItem = {{ModelFilesItem}}<ModelFilesRow> {
        show_bg: true,
        draw_bg: {
            color: #fff
        }

        cell1 = {
            spacing: 10,
            filename = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 9},
                    color: #000
                }
            }
        }

        cell2 = {
            full_size = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 9},
                    color: #000
                }
            }
        }

        cell3 = {
            spacing: 6,
            quantization_tag = <RoundedView> {
                width: Fit,
                height: Fit,
                padding: {top: 6, bottom: 6, left: 10, right: 10}

                draw_bg: {
                    instance radius: 2.0,
                    border_color: #B4B4B4,
                    border_width: 0.5,
                    color: #FFF,
                }

                quantization = <Label> {
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 9},
                        color: #000
                    }
                }
            }
            tags = <ModelFilesTags> {}
        }

        cell4 = {
            download_button = <DownloadButton> { visible: false }
            start_chat_button = <StartChatButton> { visible: false }
            download_pending_controls = <DownloadPendingControls> { visible: false }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelFileItemAction {
    Download(FileID),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelFilesItem {
    #[deref]
    view: View,

    #[rust]
    file_id: Option<FileID>,
}

impl Widget for ModelFilesItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let files_info = &scope.props.get::<FileWithDownloadInfo>().unwrap();
        let filename = &files_info.file.name;
        let size = format_model_size(&files_info.file.size).unwrap_or("-".to_string());
        let quantization = &files_info.file.quantization;
        self.apply_over(
            cx,
            live! {
                cell1 = {
                    filename = { text: (filename) }
                }
                cell2 = { full_size = { text: (size) }}
                cell3 = {
                    quantization_tag = { quantization = { text: (quantization) }}
                }
            },
        );

        if let Some(download) = &files_info.download {
            let progress = format!("{:.1}%", download.progress);
            let progress_fill_max = 74.0;
            let progress_fill = download.progress * progress_fill_max / 100.0;

            let is_resume_download_visible =
                matches!(download.status, PendingDownloadsStatus::Paused);
            let is_pause_download_visible =
                matches!(download.status, PendingDownloadsStatus::Downloading);
            let is_retry_download_visible =
                matches!(download.status, PendingDownloadsStatus::Error);
            let is_cancel_download_visible =
                !matches!(download.status, PendingDownloadsStatus::Initializing);

            let status_color = match download.status {
                PendingDownloadsStatus::Downloading | PendingDownloadsStatus::Initializing => {
                    vec3(0.035, 0.572, 0.314)
                } // #099250
                PendingDownloadsStatus::Paused => vec3(0.4, 0.44, 0.52), // #667085
                PendingDownloadsStatus::Error => vec3(0.7, 0.11, 0.09),  // #B42318
            };

            self.apply_over(
                cx,
                live! { cell4 = {
                    download_pending_controls = {
                        visible: true
                        progress_text_layout = {
                            progress_text = {
                                text: (progress)
                                draw_text: {
                                    color: (status_color)
                                }
                            }
                        }
                        progress_bar = {
                            progress_fill = {
                                width: (progress_fill)
                                draw_bg: {
                                    color: (status_color),
                                }
                            }
                        }
                        resume_download_button = {
                            visible: (is_resume_download_visible)
                        }
                        retry_download_button = {
                            visible: (is_retry_download_visible)
                        }
                        pause_download_button = {
                            visible: (is_pause_download_visible)
                        }
                        cancel_download_button = {
                            visible: (is_cancel_download_visible)
                        }
                    }
                    start_chat_button = { visible: false }
                    download_button = { visible: false }
                }},
            );
        } else if files_info.file.downloaded {
            self.apply_over(
                cx,
                live! { cell4 = {
                    download_pending_controls = { visible: false }
                    start_chat_button = { visible: true }
                    download_button = { visible: false }
                }},
            );
        } else {
            self.apply_over(
                cx,
                live! { cell4 = {
                    download_pending_controls = { visible: false }
                    start_chat_button = { visible: false }
                    download_button = { visible: true }
                }},
            );
        };

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ModelFilesItem {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();
        let Some(file_id) = self.file_id.clone() else {
            return;
        };

        if self.button(id!(download_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                ModelFileItemAction::Download(file_id.clone()),
            );
        }

        if self.button(id!(start_chat_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                ChatAction::Start(ChatHandler::Model(file_id.clone())),
            );
        }

        if [id!(resume_download_button), id!(retry_download_button)]
            .iter()
            .any(|id| self.button(*id).clicked(&actions))
        {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Play(file_id.clone()),
            );
        }

        if self.button(id!(pause_download_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Pause(file_id.clone()),
            );
        }

        if self.button(id!(cancel_download_button)).clicked(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                DownloadAction::Cancel(file_id.clone()),
            );
        }
    }
}

impl ModelFilesItemRef {
    pub fn set_file(&mut self, cx: &mut Cx, file: File) {
        let Some(mut item_widget) = self.borrow_mut() else {
            return;
        };

        item_widget.file_id = Some(file.id.clone());

        item_widget
            .model_files_tags(id!(tags))
            .set_tags(cx, &file.tags);
    }
}
