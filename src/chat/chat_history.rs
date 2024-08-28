use super::chat_history_card::{ChatHistoryCardAction, ChatHistoryCardWidgetRefExt};
use super::agent_button::AgentButtonWidgetRefExt;
use crate::data::store::Store;
use makepad_widgets::*;
use moxin_mae::MaeBackend;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import makepad_draw::shader::std::*;

    import crate::chat::shared::ChatAgentAvatar;
    import crate::chat::chat_history_card::ChatHistoryCard;
    import crate::chat::agent_button::*;

    ICON_NEW_CHAT = dep("crate://self/resources/icons/new_chat.svg")

    HeadingLabel = <Label> {
        margin: {bottom: 4},
        draw_text:{
            text_style: <REGULAR_FONT>{font_size: 10},
            color: #667085
        }
    }

    ChatHistory = {{ChatHistory}} <MoxinTogglePanel> {
        open_content = {
            <View> {
                width: Fill,
                height: Fill,
                show_bg: true
                draw_bg: {
                    color: #F2F4F7
                }

                <View> {
                    width: Fill,
                    height: Fill,

                    margin: { top: 120 }
                    padding: { left: 25, right: 25, bottom: 58 }

                    list = <PortalList> {
                        AgentHeading = <HeadingLabel> { text: "AGENTS" }
                        Agent = <AgentButton> {}
                        ChatsHeading = <HeadingLabel> { text: "CHATS", margin: {top: 10}, }
                        ChatHistoryCard = <ChatHistoryCard> {
                            padding: {top: 20}
                            cursor: Default
                        }
                    }
                }
            }
        }

        persistent_content = {
            default = {
                after = {
                    new_chat_button = <MoxinButton> {
                        width: Fit,
                        height: Fit,
                        icon_walk: {margin: { top: -1 }, width: 21, height: 21},
                        draw_icon: {
                            svg_file: (ICON_NEW_CHAT),
                            fn get_color(self) -> vec4 {
                                return #475467;
                            }
                        }
                    }
                }
            }
        }

    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatHistory {
    #[deref]
    deref: TogglePanel,
}

impl Widget for ChatHistory {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        // TODO This is a hack to redraw the chat history and reflect the
        // name change on the first message sent.
        // Maybe we should send and receive an action here?
        if let Event::Signal = event {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let agents = MaeBackend::available_agents();
        let mut chat_ids = store
            .chats
            .saved_chats
            .iter()
            .map(|c| c.borrow().id)
            .collect::<Vec<_>>();

        // Reverse sort chat ids.
        chat_ids.sort_by(|a, b| b.cmp(a));

        let agents_count = agents.len();
        let chats_count = chat_ids.len();

        // +2 for the headings.
        let items_count = agents_count + chats_count + 2;

        while let Some(view_item) = self.deref.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, items_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id == 0 {
                        let item = list.item(cx, item_id, live_id!(AgentHeading)).unwrap();
                        item.draw_all(cx, scope);
                        continue;
                    }
                    
                    let item_id = item_id - 1;

                    if item_id < agents_count {
                        let agent = &agents[item_id];
                        let item = list.item(cx, item_id, live_id!(Agent)).unwrap();
                        item.as_agent_button().set_agent(*agent);
                        item.draw_all(cx, scope);
                        continue;
                    }

                    let item_id = item_id - agents_count;

                    if item_id == 0 {
                        let item = list.item(cx, item_id, live_id!(ChatsHeading)).unwrap();
                        item.draw_all(cx, scope);
                        continue;
                    }

                    let item_id = item_id - 1;
                    
                    if item_id < chats_count {
                        let mut item = list
                            .item(cx, item_id,  live_id!(ChatHistoryCard))
                            .unwrap()
                            .as_chat_history_card();
                        let _ = item.set_chat_id(chat_ids[item_id]);
                        item.draw_all(cx, scope);
                        continue;
                    }
                }
            }
        }

        DrawStep::done()
    }
}

impl WidgetMatchEvent for ChatHistory {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        if self.button(id!(new_chat_button)).clicked(&actions) {
            store.chats.create_empty_chat();

            // Make sure text input is focused and other necessary setup happens.
            let widget_uid = self.widget_uid();
            cx.widget_action(
                widget_uid,
                &scope.path,
                ChatHistoryCardAction::ChatSelected,
            );

            self.redraw(cx);
        }

        if self.button(id!(close_panel_button)).clicked(&actions) {
            self.button(id!(close_panel_button)).set_visible(false);
            self.button(id!(open_panel_button)).set_visible(true);
            self.deref.set_open(cx, false);
        }

        if self.button(id!(open_panel_button)).clicked(&actions) {
            self.button(id!(open_panel_button)).set_visible(false);
            self.button(id!(close_panel_button)).set_visible(true);
            self.deref.set_open(cx, true);
        }
    }
}
