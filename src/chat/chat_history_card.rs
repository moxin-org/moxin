use crate::{
    data::{chats::chat::ChatID, store::Store},
    shared::portal::PortalAction,
};
use chrono::{DateTime, Local, TimeZone};

use makepad_widgets::*;

use super::chat_history_card_options::ChatHistoryCardOptionsAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::chat::shared::ChatAgentAvatar;

    ICON_DELETE = dep("crate://self/resources/icons/delete.svg")

    EditTextInput = <MoxinTextInput> {
        width: Fill,
        height: Fit,
        padding: 0,
        empty_message: ""

        draw_text: {
            text_style:<REGULAR_FONT>{height_factor: (1.3*1.3), font_size: 10},
            word: Wrap,

            instance prompt_enabled: 0.0
            fn get_color(self) -> vec4 {
                return #000;
            }
        }
    }

    EditActionButton = <MoxinButton> {
        width: 56,
        height: 31,
        spacing: 6,

        draw_bg: { color: #099250 }

        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return #fff;
            }
        }
    }

    SaveButton = <EditActionButton> {
        text: "Save"
    }

    CancelButton = <EditActionButton> {
        draw_bg: { border_color: #D0D5DD, border_width: 1.0, color: #fff }

        draw_text: {
            text_style: <REGULAR_FONT>{font_size: 9},
            fn get_color(self) -> vec4 {
                return #000;
            }
        }
        text: "Cancel"
    }

    ChatHistoryCard = {{ChatHistoryCard}} {
        content = <RoundedView> {
            flow: Down
            width: 248
            height: Fit
            padding: 20
            spacing: 12

            cursor: Hand

            draw_bg: {
                color: #fff
                border_width: 1
            }

            <View> {
                width: Fill
                height: Fit
                flow: Right
                spacing: 10
                padding: { top: 4, bottom: 4 }
                margin: 0

                title_wrapper = <RoundedView> {
                    show_bg: true,
                    draw_bg: {
                        radius: 12.0,
                    },

                    width: Fill,
                    height: Fit,
                    flow: Down,
                    align: {x: 0.5, y: 0.0},

                    title_input_container = <View> {
                        visible: false,
                        width: Fill,
                        height: Fit,
                        title_input = <EditTextInput> {}
                    }

                    title_label_container = <View> {
                        visible: false,
                        width: Fill,
                        height: Fit,

                        title_label = <Label> {
                            width: Fill,
                            height: Fit,
                            draw_text:{
                                text_style: <BOLD_FONT>{font_size: 10},
                                color: #000,
                            }
                            text: ""
                        }
                    }

                    edit_buttons = <View> {
                        visible: false,
                        width: Fit,
                        height: Fit,
                        margin: {top: 10},
                        spacing: 6,
                        save = <SaveButton> {}
                        cancel = <CancelButton> {}
                    }
                }

                // TODO: This is horrible, find a way of getting the position of the button.
                chat_options_wrapper = <View> {
                    width: Fit
                    height: Fit
                    padding: 4

                    chat_options = <MoxinButton> {
                        width: Fit
                        height: Fit
                        padding: {top: 0, right: 4, bottom: 6, left: 4}
                        margin: { top: -4}

                        draw_bg: {
                            radius: 5
                        }

                        draw_text:{
                            text_style: <BOLD_FONT>{font_size: 14},
                            color: #667085,
                        }
                        text: "..."
                    }
                }
            }

            <View> {
                width: Fill
                height: Fit
                align: {y: 1}

                avatar = <ChatAgentAvatar> {
                    width: 30
                    height: 30

                    draw_bg: {
                        radius: 8
                    }
                    avatar_label = {
                        text: ""
                    }
                }

                filler = <View> {width: Fill}

                date = <Label> {
                    width: Fit,
                    height: Fit,
                    draw_text:{
                        text_style: <REGULAR_FONT>{font_size: 10},
                        color: #667085,
                    }
                    text: "5:29 PM, 5/12/24"
                }
            }


        }
    }
}

#[derive(Default, Debug, PartialEq)]
enum TitleState {
    OnEdit,
    #[default]
    Editable,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatHistoryCard {
    #[deref]
    view: View,
    #[rust]
    chat_id: ChatID,

    #[rust]
    title_edition_state: TitleState,
}

impl Widget for ChatHistoryCard {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get_mut::<Store>().unwrap();
        let chat = store
            .chats
            .saved_chats
            .iter()
            .find(|c| c.borrow().id == self.chat_id)
            .unwrap();

        if let Some(current_chat_id) = store.chats.get_current_chat_id() {
            let content_view = self.view(id!(content));

            if current_chat_id == self.chat_id {
                let active_border_color = vec3(0.082, 0.522, 0.604);
                content_view.apply_over(
                    cx,
                    live! {
                        draw_bg: {border_color: (active_border_color)}
                    },
                );
            } else {
                let border_color = vec3(0.918, 0.925, 0.941);
                content_view.apply_over(
                    cx,
                    live! {
                        draw_bg: {border_color: (border_color)}
                    },
                );
            }
        }

        self.set_title_text(chat.borrow_mut().get_title());
        self.update_title_visibility(cx);

        let initial_letter = store
            .get_last_used_file_initial_letter(self.chat_id)
            .unwrap_or('A')
            .to_uppercase()
            .to_string();

        let avatar_label = self.view.label(id!(avatar.avatar_label));
        avatar_label.set_text(&initial_letter);

        let date_label = self.view.label(id!(date));

        // Format date.
        // TODO: Feels wrong to asume the id will always be the date, do smth about this.
        let datetime =
            DateTime::from_timestamp_millis(chat.borrow().id as i64).expect("Invalid timestamp");
        let local_datetime: DateTime<Local> = Local.from_utc_datetime(&datetime.naive_utc());
        let formatted_date = local_datetime.format("%-I:%M %p, %-d/%m/%y").to_string();

        date_label.set_text(&formatted_date);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatHistoryCard {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();

        match self.title_edition_state {
            TitleState::Editable => self.handle_title_editable_actions(cx, actions, scope),
            TitleState::OnEdit => self.handle_title_on_edit_actions(cx, actions, scope),
        }

        let chat_options_wrapper_rect = self.view(id!(chat_options_wrapper)).area().rect(cx);
        let chat_options_button = self.button(id!(chat_options));
        if chat_options_button.clicked(actions) {
            let cords = chat_options_wrapper_rect.pos;
            let cords = dvec2(cords.x, cords.y + chat_options_wrapper_rect.size.y);

            cx.widget_action(
                widget_uid,
                &scope.path,
                ChatHistoryCardOptionsAction::Selected(self.chat_id, cords),
            );
            cx.widget_action(
                widget_uid,
                &scope.path,
                PortalAction::ShowPortalView(live_id!(chat_history_card_options_portal_view)),
            );

            chat_options_button.reset_hover(cx);
            return;
        }

        if let Some(fe) = self.view(id!(content)).finger_down(actions) {
            if fe.tap_count == 1 {
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    ChatHistoryCardAction::ChatSelected(self.chat_id),
                );
                let store = scope.data.get_mut::<Store>().unwrap();
                store.select_chat(self.chat_id);
                self.redraw(cx);
            }
        }
    }
}

impl ChatHistoryCard {
    pub fn set_chat_id(&mut self, id: ChatID) {
        self.chat_id = id;
    }

    fn set_title_text(&mut self, text: &str) {
        self.view.label(id!(title_label)).set_text(text.trim());
        if let TitleState::Editable = self.title_edition_state {
            self.view.text_input(id!(title_input)).set_text(text.trim());
        }
    }

    fn update_title_visibility(&mut self, cx: &mut Cx) {
        let on_edit = matches!(self.title_edition_state, TitleState::OnEdit);
        self.view(id!(edit_buttons)).set_visible(on_edit);
        self.view(id!(title_input_container)).set_visible(on_edit);

        let editable = matches!(self.title_edition_state, TitleState::Editable);
        self.view(id!(title_label_container)).set_visible(editable);

        self.redraw(cx);
    }

    fn transition_title_state(&mut self, cx: &mut Cx) {
        self.title_edition_state = match self.title_edition_state {
            TitleState::OnEdit => TitleState::Editable,
            TitleState::Editable => TitleState::OnEdit,
        };

        self.update_title_visibility(cx);
    }

    pub fn handle_title_editable_actions(
        &mut self,
        cx: &mut Cx,
        actions: &Actions,
        _scope: &mut Scope,
    ) {
        for action in actions {
            if let ChatHistoryCardAction::ActivateTitleEdition(chat_id) =
                action.as_widget_action().cast::<ChatHistoryCardAction>()
            {
                if chat_id == self.chat_id {
                    self.transition_title_state(cx);
                }
            }
        }
    }

    fn handle_title_on_edit_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let save_button = self.button(id!(save));
        if save_button.clicked(actions) {
            let updated_title = self.text_input(id!(title_input)).text();
            let chat = store
                .chats
                .saved_chats
                .iter()
                .find(|c| c.borrow().id == self.chat_id)
                .unwrap();

            if !updated_title.trim().is_empty() && chat.borrow().get_title() != updated_title {
                chat.borrow_mut().set_title(updated_title.clone());
                chat.borrow().save();
            }

            save_button.reset_hover(cx);
            self.transition_title_state(cx)
        }

        let cancel_button = self.button(id!(cancel));
        if cancel_button.clicked(actions) {
            cancel_button.reset_hover(cx);
            self.transition_title_state(cx)
        }
    }
}

impl ChatHistoryCardRef {
    pub fn set_chat_id(&mut self, id: ChatID) -> Result<(), &'static str> {
        let Some(mut inner) = self.borrow_mut() else {
            return Err("Widget not found in the document");
        };

        inner.set_chat_id(id);
        Ok(())
    }
}

#[derive(Clone, DefaultNone, Eq, Hash, PartialEq, Debug)]
pub enum ChatHistoryCardAction {
    None,
    ChatSelected(ChatID),
    ActivateTitleEdition(ChatID),
}
