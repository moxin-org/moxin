use makepad_widgets::*;
use moxin_protocol::data::FileID;
use std::cell::{Ref, RefCell, RefMut};

use crate::{
    chat::{
        chat_line::{ChatLineAction, ChatLineWidgetRefExt},
        model_selector::ModelSelectorWidgetExt,
        model_selector_list::ModelSelectorAction,
    },
    data::{
        chats::chat::{Chat, ChatMessage},
        store::Store,
    },
    shared::actions::ChatAction,
};

use super::chat_history_card::ChatHistoryCardAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import makepad_draw::shader::std::*;

    import crate::chat::model_selector::ModelSelector;
    import crate::chat::chat_line::ChatLine;
    import crate::chat::shared::ChatAgentAvatar;

    ICON_PROMPT = dep("crate://self/resources/icons/prompt.svg")
    ICON_STOP = dep("crate://self/resources/icons/stop.svg")
    ICON_JUMP_TO_BOTTOM = dep("crate://self/resources/icons/jump_to_bottom.svg")

    CircleButton = <MoxinButton> {
        padding: {right: 4},
        margin: {bottom: 2},

        draw_icon: {
            color: #fff
        }
        icon_walk: {width: 12, height: 12}
    }

    UserChatLine = <ChatLine> {
        margin: {left: 100}
        avatar_section = {
            visible: false,
        }
        main_section = {
            body_section = {
                align: {x: 1.0, y: 0.5},
                sender_name_layout = {
                    visible: false,
                }
                bubble = {
                    draw_bg: {
                        color: #15859A
                    }
                    markdown_message_container = {
                        markdown_message = {
                            draw_normal: {
                                color: (#fff),
                            }
                            draw_italic: {
                                color: (#fff),
                            }
                            draw_bold: {
                                color: (#fff),
                            }
                            draw_bold_italic: {
                                color: (#fff),
                            }
                            draw_fixed: {
                                color: (#fff),
                            }
                            draw_block: {
                                line_color: (#fff)
                                sep_color: (#12778a)
                                quote_bg_color: (#12778a)
                                quote_fg_color: (#106a7b)
                                block_color: (#12778a)
                                code_color: (#12778a)
                            }
                        }
                    }
                    plain_text_message_container = {
                        plain_text_message = {
                            draw_text: {
                                color: #fff
                            }
                        }
                    }
                    edit_buttons = {
                        save = {
                            draw_bg: { border_color: #D0D5DD, border_width: 1.0, color: #fff }
                            draw_text: {
                                fn get_color(self) -> vec4 {
                                    return #099250;
                                }
                            }
                        }
                        save_and_regenerate = {
                            draw_bg: { border_color: #D0D5DD, border_width: 1.0, color: #fff }
                            draw_text: {
                                fn get_color(self) -> vec4 {
                                    return #099250;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ModelChatLine = <ChatLine> {
        avatar_section = {
            <ChatAgentAvatar> {}
        }
        main_section = {
            body_section = {
                bubble = {
                    padding: {left: 0, bottom: 0, top: 0}
                    markdown_message_container = {
                        markdown_message = {
                            draw_normal: {
                                color: (#000),
                            }
                            draw_italic: {
                                color: (#000),
                            }
                            draw_bold: {
                                color: (#000),
                            }
                            draw_bold_italic: {
                                color: (#000),
                            }
                            draw_fixed: {
                                color: (#000),
                            }
                            draw_block: {
                                line_color: (#000)
                                sep_color: (#EDEDED)
                                quote_bg_color: (#EDEDED)
                                quote_fg_color: (#969696)
                                block_color: (#EDEDED)
                                code_color: (#EDEDED)
                            }
                        }
                    }
                }
            }
        }
    }

    JumpToBottom = <View> {
        width: Fill,
        height: Fill,
        align: {x: 1.0, y: 1.0},
        padding: {bottom: 60},

        jump_to_bottom = <CircleButton> {
            width: 34,
            height: 34,
            margin: {bottom: 10},

            draw_bg: {
                radius: 8.0,
                color: #fff,
                border_width: 1.0,
                border_color: #EAECF0,
            }
            draw_icon: {
                svg_file: (ICON_JUMP_TO_BOTTOM),
                fn get_color(self) -> vec4 {
                    return #1C1B1F;
                }
            }
            icon_walk: {
                margin: {top: 6, left: -4},
            }
        }
    }

    PromptButton = <CircleButton> {
        width: 28,
        height: 28,

        draw_bg: {
            radius: 6.5,
            color: #D0D5DD
        }
        icon_walk: {
            margin: {top: 0, left: -4},
        }
    }

    ChatPromptInput = <RoundedView> {
        width: Fill,
        height: Fit,

        show_bg: true,
        draw_bg: {
            color: #fff
        }

        padding: {top: 6, bottom: 6, left: 4, right: 10}

        spacing: 4,
        align: {x: 0.0, y: 1.0},

        draw_bg: {
            radius: 10.0,
            border_color: #D0D5DD,
            border_width: 1.0,
        }

        prompt = <MoxinTextInput> {
            width: Fill,
            height: Fit,

            empty_message: "Enter a message"
            draw_bg: {
                radius: 30.0
                color: #fff
            }
            draw_text: {
                text_style:<REGULAR_FONT>{font_size: 10},

                instance prompt_enabled: 0.0
                fn get_color(self) -> vec4 {
                    return mix(
                        #98A2B3,
                        #000,
                        self.prompt_enabled
                    )
                }
            }
        }

        prompt_send_button = <PromptButton> {
            draw_icon: {
                svg_file: (ICON_PROMPT),
            }
        }

        prompt_stop_button = <PromptButton> {
            draw_icon: {
                svg_file: (ICON_STOP),
            }
        }
    }

    ChatPanel = {{ChatPanel}} {
        flow: Overlay
        width: Fill
        height: Fill

        <View> {
            flow: Overlay
            width: Fill
            height: Fill
            padding: {left: 25, right: 25, bottom: 20},

            no_downloaded_model = <View> {
                visible: false,
                width: Fill,
                height: Fill,

                flow: Down,
                align: {x: 0.5, y: 0.5},

                <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    spacing: 30,
                    align: {x: 0.5, y: 0.5},

                    <Label> {
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 12},
                            color: #667085
                        }
                        text: "You haven’t downloaded any models yet."
                    }
                    go_to_discover_button = <MoxinButton> {
                        width: Fit,
                        height: Fit,

                        draw_bg: {
                            border_color: #D0D5DD,
                            border_width: 1.0,
                            color: #fff,
                            color_hover: #E2F1F1,
                            radius: 2.0,
                        }

                        padding: {top: 14, right: 12, bottom: 14, left: 12}
                        text: "Go To Discover"
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 12},
                            fn get_color(self) -> vec4 {
                                return #087443;
                            }
                        }
                    }
                }

                <View> {
                    width: Fill, height: Fit
                    flow: Down,
                    align: {x: 0.5, y: 0.5},
                    no_downloaded_model_prompt_input = <ChatPromptInput> {}
                }

            }

            no_model = <View> {
                visible: false,
                width: Fill,
                height: Fill,

                flow: Down,
                align: {x: 0.5, y: 0.5},

                <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,
                    spacing: 30,
                    align: {x: 0.5, y: 0.5},

                    <Icon> {
                        draw_icon: {
                            svg_file: dep("crate://self/resources/icons/chat.svg"),
                            color: #D0D5DD
                        }
                        icon_walk: {width: 128, height: 128}
                    }

                    <Label> {
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 14},
                            color: #667085
                        }
                        text: "Start chatting by choosing a model from above"
                    }
                }

                <View> {
                    width: Fill, height: Fit
                    flow: Down,
                    align: {x: 0.5, y: 0.5},
                    no_model_prompt_input = <ChatPromptInput> {}
                }

            }

            empty_conversation = <View> {
                visible: false,

                width: Fill,
                height: Fill,

                flow: Down,
                spacing: 30,
                align: {x: 0.5, y: 0.5},

                <ChatAgentAvatar> {}
                <Label> {
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 14},
                        color: #101828
                    }
                    text: "How can I help you?"
                }
            }

            main = <View> {
                visible: false

                width: Fill,
                height: Fill,

                margin: { top: 86 }
                spacing: 4,
                flow: Down,

                <View> {
                    width: Fill,
                    height: Fill,

                    flow: Overlay
                    chat = <PortalList> {
                        scroll_bar: {
                            bar_size: 0.0,
                        }
                        width: Fill,
                        height: Fill,

                        drag_scrolling: false,

                        UserChatLine = <UserChatLine> {}
                        ModelChatLine = <ModelChatLine> {}
                        EndOfChat = <View> {height: 0.1}
                    }
                }

                main_prompt_input = <ChatPromptInput> {}
            }

            model_selector = <ModelSelector> {}
        }

        <JumpToBottom> {}
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum State {
    /// `Unknown` is simply the default state, meaning the state has not been loaded yet,
    /// and therefore indicates a development error if it is encountered.
    #[default]
    Unknown,
    NoModelsAvailable,
    NoModelSelected,
    ModelSelectedWithEmptyChat {
        is_loading: bool,
    },
    ModelSelectedWithChat {
        is_loading: bool,
        sticked_to_bottom: bool,
        is_streaming: bool,
    },
}

enum PromptInputMode {
    Enabled,
    Disabled,
}
enum PromptInputButton {
    Send,
    Stop,
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatPanel {
    #[deref]
    view: View,

    #[rust]
    state: State,

    #[rust]
    portal_list_end_reached: bool,
}

impl Widget for ChatPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
        self.update_state(scope);

        if let Event::Signal = event {
            match self.state {
                State::ModelSelectedWithChat {
                    is_streaming: true,
                    sticked_to_bottom,
                    ..
                } => {
                    if sticked_to_bottom {
                        self.scroll_messages_to_bottom(cx);
                    }

                    // Redraw because we expect to see new or updated chat entries
                    self.redraw(cx);
                }
                _ => {}
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.update_view(cx, scope);

        while let Some(view_item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                self.draw_messages(cx, scope, &mut list);
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatPanel {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let widget_uid = self.widget_uid();
        let store = scope.data.get_mut::<Store>().unwrap();

        for action in actions
            .iter()
            .filter_map(|action| action.as_widget_action())
        {
            if let ChatHistoryCardAction::ChatSelected(_) = action.cast() {
                self.reset_scroll_messages(&store);
                self.redraw(cx);
            }

            if let ModelSelectorAction::Selected(downloaded_file) = action.cast() {
                store.load_model(downloaded_file.file);
                self.redraw(cx)
            }

            match action.cast() {
                ChatAction::Start(file_id) => {
                    let downloaded_file = store
                        .downloads
                        .downloaded_files
                        .iter()
                        .find(|file| file.file.id == file_id)
                        .expect("Attempted to start chat with a no longer existing file")
                        .clone();

                    store
                        .chats
                        .create_empty_chat_and_load_file(downloaded_file.file);
                }
                _ => {}
            }

            match action.cast() {
                ChatLineAction::Delete(id) => {
                    store.chats.delete_chat_message(id);
                    self.redraw(cx);
                }
                ChatLineAction::Edit(id, updated, regenerate) => {
                    store.chats.edit_chat_message(id, updated, regenerate);
                    self.redraw(cx);
                }
                _ => {}
            }

            if let ChatPanelAction::UnloadIfActive(file_id) = action.cast() {
                if store
                    .chats
                    .loaded_model
                    .as_ref()
                    .map_or(false, |file| file.id == file_id)
                {
                    store.chats.eject_model().expect("Failed to eject model");
                    self.unload_model(cx);
                }
            }
        }

        if self.button(id!(jump_to_bottom)).clicked(actions) {
            self.scroll_messages_to_bottom(cx);
            self.redraw(cx);
        }

        match self.state {
            State::ModelSelectedWithChat {
                is_streaming: false,
                ..
            }
            | State::ModelSelectedWithEmptyChat { .. } => {
                self.handle_prompt_input_actions(cx, actions, scope);
            }
            State::ModelSelectedWithChat {
                is_streaming: true, ..
            } => {
                if self
                    .button(id!(main_prompt_input.prompt_stop_button))
                    .clicked(actions)
                {
                    store.chats.cancel_chat_streaming();
                }
            }
            _ => {}
        }

        if self
            .button(id!(no_downloaded_model.go_to_discover_button))
            .clicked(actions)
        {
            cx.widget_action(widget_uid, &scope.path, ChatPanelAction::NavigateToDiscover);
        }
    }
}

impl ChatPanel {
    fn update_state(&mut self, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();
        //let loader = &store.chats.model_loader;

        self.state = if store.downloads.downloaded_files.is_empty() {
            State::NoModelsAvailable
        } else if store.chats.loaded_model.is_none() {
            State::NoModelSelected
        } else {
            let is_loading = store.chats.get_currently_loading_model().is_some();

            store.chats.get_current_chat().map_or(
                State::ModelSelectedWithEmptyChat { is_loading },
                |chat| {
                    if chat.borrow().messages.is_empty() {
                        State::ModelSelectedWithEmptyChat { is_loading }
                    } else {
                        State::ModelSelectedWithChat {
                            is_loading,
                            sticked_to_bottom: self.portal_list_end_reached
                                || !matches!(self.state, State::ModelSelectedWithChat { .. }),
                            is_streaming: chat.borrow().is_streaming,
                        }
                    }
                },
            )
        };
    }

    fn update_prompt_input(&mut self, cx: &mut Cx) {
        match self.state {
            State::ModelSelectedWithEmptyChat { is_loading: true }
            | State::ModelSelectedWithChat {
                is_loading: true, ..
            } => {
                self.activate_prompt_input(cx, PromptInputMode::Disabled, PromptInputButton::Send);
            }
            State::ModelSelectedWithEmptyChat { is_loading: false }
            | State::ModelSelectedWithChat {
                is_loading: false,
                is_streaming: false,
                ..
            } => {
                self.activate_prompt_input(cx, PromptInputMode::Enabled, PromptInputButton::Send);
            }
            State::ModelSelectedWithChat {
                is_streaming: true, ..
            } => {
                self.activate_prompt_input(cx, PromptInputMode::Disabled, PromptInputButton::Stop);
            }
            _ => {
                // Input prompts should not be visible in other conditions
            }
        }
    }

    fn activate_prompt_input(
        &mut self,
        cx: &mut Cx,
        mode: PromptInputMode,
        button: PromptInputButton,
    ) {
        let prompt_input = self.text_input(id!(main_prompt_input.prompt));

        let enabled = match mode {
            PromptInputMode::Enabled => !prompt_input.text().is_empty(),
            PromptInputMode::Disabled => false,
        };

        let (button_color, prompt_enabled) = if enabled {
            (vec3(0.0, 0.0, 0.0), 1.0)
        } else {
            // The color code is #D0D5DD
            (vec3(0.816, 0.835, 0.867), 0.0)
        };

        prompt_input.apply_over(
            cx,
            live! {
                draw_text: { prompt_enabled: (prompt_enabled) }
            },
        );

        let send_button = self.button(id!(main_prompt_input.prompt_send_button));
        let stop_button = self.button(id!(main_prompt_input.prompt_stop_button));
        match button {
            PromptInputButton::Send => {
                // The send button is enabled or not based on the prompt input
                send_button.set_visible(true);
                send_button.set_enabled(enabled);
                send_button.apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            color: (button_color)
                        }
                    },
                );
                stop_button.set_visible(false);
            }
            PromptInputButton::Stop => {
                // The stop button is always enabled, when visible
                stop_button.set_visible(true);
                stop_button.set_enabled(true);
                stop_button.apply_over(
                    cx,
                    live! {
                        draw_bg: {
                            color: #x000
                        }
                    },
                );
                send_button.set_visible(false);
            }
        }
    }

    fn scroll_messages_to_bottom(&mut self, cx: &mut Cx) {
        let mut list = self.portal_list(id!(chat));
        list.smooth_scroll_to_end(cx, 10, 80.0);
    }

    fn reset_scroll_messages(&mut self, store: &Store) {
        let list = self.portal_list(id!(chat));
        let messages = get_chat_messages(store).unwrap();
        let index = messages.len().saturating_sub(1);
        list.set_first_id(index);
    }

    fn unload_model(&mut self, cx: &mut Cx) {
        self.model_selector(id!(model_selector)).deselect(cx);
        self.view.redraw(cx);
    }

    fn handle_prompt_input_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let prompt_input = self.text_input(id!(main_prompt_input.prompt));
        if let Some(_text) = prompt_input.changed(actions) {
            self.redraw(cx);
        }

        if self
            .button(id!(main_prompt_input.prompt_send_button))
            .clicked(&actions)
        {
            self.send_message(cx, scope, prompt_input.text());
        }

        if let Some(prompt) = prompt_input.returned(actions) {
            self.send_message(cx, scope, prompt);
        }
    }

    fn send_message(&mut self, cx: &mut Cx, scope: &mut Scope, prompt: String) {
        // Check if we have any text to send
        if prompt.trim().is_empty() {
            return;
        }

        // Let's confirm we're in an appropriate state to send a message
        self.update_state(scope);
        if matches!(
            self.state,
            State::ModelSelectedWithChat {
                is_streaming: false,
                is_loading: false,
                ..
            } | State::ModelSelectedWithEmptyChat { is_loading: false }
        ) {
            let store = scope.data.get_mut::<Store>().unwrap();
            store.chats.send_chat_message(prompt.clone());

            let prompt_input = self.text_input(id!(main_prompt_input.prompt));
            prompt_input.set_text_and_redraw(cx, "");
            prompt_input.set_cursor(0, 0);

            // Scroll to the bottom when the message is sent
            self.scroll_messages_to_bottom(cx);
            self.redraw(cx);
        }
    }

    fn update_view(&mut self, cx: &mut Cx2d, scope: &mut Scope) {
        self.update_visibilities();
        self.update_prompt_input(cx);

        match self.state {
            State::ModelSelectedWithEmptyChat { .. } => {
                let store = scope.data.get::<Store>().unwrap();

                self.view(id!(empty_conversation))
                    .label(id!(avatar_label))
                    .set_text(&get_model_initial_letter(store).unwrap_or('A').to_string());
            }
            _ => {}
        }
    }

    fn update_visibilities(&mut self) {
        let empty_conversation = self.view(id!(empty_conversation));
        let jump_to_bottom = self.button(id!(jump_to_bottom));
        let main = self.view(id!(main));
        let no_downloaded_model = self.view(id!(no_downloaded_model));
        let no_model = self.view(id!(no_model));

        match self.state {
            State::NoModelsAvailable => {
                empty_conversation.set_visible(false);
                jump_to_bottom.set_visible(false);
                main.set_visible(false);
                no_model.set_visible(false);

                no_downloaded_model.set_visible(true);
            }
            State::NoModelSelected => {
                empty_conversation.set_visible(false);
                jump_to_bottom.set_visible(false);
                main.set_visible(false);
                no_downloaded_model.set_visible(false);

                no_model.set_visible(true);
            }
            State::ModelSelectedWithEmptyChat { .. } => {
                jump_to_bottom.set_visible(false);
                no_downloaded_model.set_visible(false);
                no_model.set_visible(false);

                empty_conversation.set_visible(true);
                main.set_visible(true);
            }
            State::ModelSelectedWithChat {
                sticked_to_bottom, ..
            } => {
                empty_conversation.set_visible(false);
                no_downloaded_model.set_visible(false);
                no_model.set_visible(false);

                main.set_visible(true);
                jump_to_bottom.set_visible(!sticked_to_bottom);
            }
            _ => {}
        }
    }

    fn draw_messages(&mut self, cx: &mut Cx2d, scope: &mut Scope, list: &mut RefMut<PortalList>) {
        let store = scope.data.get::<Store>().unwrap();
        let messages = get_chat_messages(store).unwrap();
        let messages_count = messages.len();

        self.portal_list_end_reached = false;
        list.set_item_range(cx, 0, messages_count + 1);
        while let Some(item_id) = list.next_visible_item(cx) {
            if item_id < messages_count {
                let chat_line_data = &messages[item_id];

                let item;
                let mut chat_line_item;
                if chat_line_data.is_assistant() {
                    item = list.item(cx, item_id, live_id!(ModelChatLine)).unwrap();
                    chat_line_item = item.as_chat_line();

                    let username = chat_line_data.username.as_ref().map_or("", String::as_str);
                    chat_line_item.set_sender_name(&username);
                    chat_line_item.set_regenerate_button_visible(false);
                    chat_line_item
                        .set_avatar_text(&get_initial_letter(username).unwrap().to_string());
                } else {
                    item = list.item(cx, item_id, live_id!(UserChatLine)).unwrap();
                    chat_line_item = item.as_chat_line();
                    chat_line_item.set_regenerate_button_visible(true);
                };

                chat_line_item.set_message_text(cx, &chat_line_data.content);
                chat_line_item.set_message_id(chat_line_data.id);

                // Disable actions for the last chat line when model is streaming
                if matches!(
                    self.state,
                    State::ModelSelectedWithChat {
                        is_streaming: true,
                        ..
                    }
                ) && item_id == messages_count - 1
                {
                    chat_line_item.set_actions_enabled(cx, false);
                } else {
                    chat_line_item.set_actions_enabled(cx, true);
                }

                item.draw_all(cx, &mut Scope::empty());
            } else {
                self.portal_list_end_reached = true;
                let item = list.item(cx, item_id, live_id!(EndOfChat)).unwrap();
                item.draw_all(cx, &mut Scope::empty());
            }
        }
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ChatPanelAction {
    UnloadIfActive(FileID),
    NavigateToDiscover,
    None,
}

fn get_chat(store: &Store) -> Option<&RefCell<Chat>> {
    store.chats.get_current_chat()
}

fn get_initial_letter(word: &str) -> Option<char> {
    word.chars().next()
}

fn get_model_initial_letter(store: &Store) -> Option<char> {
    let chat = get_chat(store)?;
    let initial_letter = store.get_last_used_file_initial_letter(chat.borrow().id)?;
    Some(initial_letter.to_ascii_uppercase())
}

fn get_chat_messages(store: &Store) -> Option<Ref<Vec<ChatMessage>>> {
    get_chat(store).map(|chat| Ref::map(chat.borrow(), |chat| &chat.messages))
}
