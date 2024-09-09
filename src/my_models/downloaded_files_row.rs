use super::{delete_model_modal::DeleteModelModalAction, model_info_modal::ModelInfoModalAction};
use crate::shared::actions::ChatHandler;
use crate::shared::modal::ModalWidgetExt;
use crate::shared::utils::format_model_size;
use crate::shared::{actions::ChatAction, utils::human_readable_name};
use makepad_widgets::*;
use moxin_protocol::data::{DownloadedFile, FileID};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::modal::*;
    import crate::my_models::model_info_modal::ModelInfoModal;
    import crate::my_models::delete_model_modal::DeleteModelModal;

    ICON_START_CHAT = dep("crate://self/resources/icons/start_chat.svg")
    ICON_INFO = dep("crate://self/resources/icons/info.svg")
    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")
    MODEL_CTA_COLOR = #127487

    DownloadedFilesRowButton = <MoxinButton> {
        height: 40

        draw_bg: {
            border_color: #ccc,
        }

        draw_icon: {
            color: #087443;
        }
    }

    ModelFile = <View> {
        flow: Down
        width: 600

        h_wrapper = <View> {
            flow: Right
            width: Fill
            spacing: 15
            name_tag = <View> {
                width: Fit
                align: {x: 0.0, y: 0.5}
                name = <Label> {
                    width: Fit
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 9}
                        color: #x0
                    }
                }
            }

            base_model_tag = <View> {
                width: Fit
                align: {x: 0.0, y: 0.5}
                base_model = <AttributeTag> {
                    draw_bg: { color: #F0D6F5 },
                }
            }
            parameters_tag = <View> {
                width: Fit
                align: {x: 0.0, y: 0.5}
                parameters = <AttributeTag> {
                    draw_bg: { color: #D4E6F7 },
                }
            }
        }
        model_version_tag = <View> {
            width: Fit
            align: {x: 0.0, y: 0.5}
            version = <Label> {
                width: Fit
                draw_text: {
                    wrap: Ellipsis
                    text_style: <REGULAR_FONT>{font_size: 9}
                    color: #667085
                }
            }
        }
    }

    DownloadedFilesTag = <View> {
        width: 100
        align: {x: 0.0, y: 0.5}
        file_size = <Label> {
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9}
                color: #x0
            }
        }
    }

    RowActions = <View> {
        width: 250
        flow: Right
        spacing: 10
        align: {x: 0.0, y: 0.5}

        start_chat_button = <DownloadedFilesRowButton> {
            width: 140
            text: "Chat with Model",
            draw_text: {
                color: (MODEL_CTA_COLOR)
                text_style: <REGULAR_FONT>{font_size: 9}
            }
            draw_icon: {
                svg_file: (ICON_START_CHAT)
                color: (MODEL_CTA_COLOR)
            }
        }

        <View> { width: Fill, height: Fit }

        info_button = <DownloadedFilesRowButton> {
            width: 40
            draw_icon: {
                svg_file: (ICON_INFO),
                color: #0099FF
            }
        }

        delete_button = <DownloadedFilesRowButton> {
            width: 40
            draw_icon: {
                svg_file: (ICON_DELETE),
                color: #B42318
            }
        }
    }


    DownloadedFilesRow = {{DownloadedFilesRow}} {
        // This is necesary because we have a Modal widget inside this widget
        flow: Overlay,
        width: Fill,
        height: Fit,

        <View> {
            // Heads-up: rows break the Portal List without fixed height
            height: 85,
            flow: Down
            width: Fill
            align: {x: 0.0, y: 0.5}

            show_bg: true
            draw_bg: {
                color: #FFF;
            }

            separator_line = <Line> {}
            h_wrapper = <View> {
                flow: Right
                width: Fit
                padding: {top: 10, bottom: 10, left: 20, right: 20}
                spacing: 30
                show_bg: true
                draw_bg: {
                    color: #FFF;
                }

                model_file = <ModelFile> {}
                file_size_tag = <DownloadedFilesTag> {}
                date_added_tag = <DownloadedFilesTag> {}
                row_actions = <RowActions> {}
            }
        }

        info_modal = <Modal> {
            content: {
                <ModelInfoModal> {}
            }
        }

        delete_modal = <Modal> {
            content: {
                <DeleteModelModal> {}
            }
        }
    }
}

pub struct DownloadedFilesRowProps {
    pub downloaded_file: DownloadedFile,
}

#[derive(Live, LiveHook, Widget)]
pub struct DownloadedFilesRow {
    #[deref]
    view: View,

    #[rust]
    file_id: Option<FileID>,
}

impl Widget for DownloadedFilesRow {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let props = scope.props.get::<DownloadedFilesRowProps>().unwrap();
        let downloaded_file = &props.downloaded_file;

        // Name tag
        let name = human_readable_name(&downloaded_file.file.name);
        self.label(id!(h_wrapper.model_file.h_wrapper.name_tag.name))
            .set_text(&name);

        // Base model tag
        let base_model = dash_if_empty(&downloaded_file.model.architecture);
        self.label(id!(h_wrapper
            .model_file
            .base_model_tag
            .base_model
            .attr_name))
            .set_text(&base_model);

        // Parameters tag
        let parameters = dash_if_empty(&downloaded_file.model.size);
        self.label(id!(h_wrapper
            .model_file
            .parameters_tag
            .parameters
            .attr_name))
            .set_text(&parameters);

        // Version tag
        let filename = format!(
            "{}/{}",
            downloaded_file.model.name, downloaded_file.file.name
        );
        self.label(id!(h_wrapper.model_file.model_version_tag.version))
            .set_text(&filename);

        // File size tag
        let file_size = format_model_size(&downloaded_file.file.size).unwrap_or("-".to_string());
        self.label(id!(h_wrapper.file_size_tag.file_size))
            .set_text(&file_size);

        // Added date tag
        let formatted_date = downloaded_file.downloaded_at.format("%d/%m/%Y").to_string();
        self.label(id!(h_wrapper.date_added_tag.date_added))
            .set_text(&formatted_date);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for DownloadedFilesRow {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        if self.button(id!(start_chat_button)).clicked(actions) {
            if let Some(file_id) = &self.file_id {
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    ChatAction::Start(ChatHandler::Model(file_id.clone())),
                );
            }
        }

        if self.button(id!(row_actions.info_button)).clicked(actions) {
            self.modal(id!(info_modal)).open(cx);
        }

        if self.button(id!(row_actions.delete_button)).clicked(actions) {
            self.modal(id!(delete_modal)).open(cx);
        }

        for action in actions {
            if matches!(
                action.as_widget_action().cast(),
                DeleteModelModalAction::ModelDeleted
                    | DeleteModelModalAction::Cancelled
                    | DeleteModelModalAction::CloseButtonClicked
            ) {
                self.modal(id!(delete_modal)).close(cx);
            }

            if let ModelInfoModalAction::CloseButtonClicked = action.as_widget_action().cast() {
                self.modal(id!(info_modal)).close(cx);
            }
        }
    }
}

impl DownloadedFilesRowRef {
    pub fn set_file_id(&mut self, file_id: FileID) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.file_id = Some(file_id);
    }
}

fn dash_if_empty(input: &str) -> &str {
    if input.is_empty() {
        "-"
    } else {
        input
    }
}
