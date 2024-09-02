use makepad_widgets::*;
use moxin_mae::MaeAgent;

use crate::data::store::Store;

use super::{chat_history_card::ChatHistoryCardAction, prompt_input::PromptInputAction, shared::ChatAgentAvatarWidgetExt};

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::chat::shared::ChatAgentAvatar;

    AgentButton = {{AgentButton}} {
        flow: Overlay,
        align: { x: 0.0, y: 0.5 },
        agent_avatar = <ChatAgentAvatar> {
            padding: { left: 9 },
        }
        button = <MoxinButton> {
            flow: Right,
            align: { x: 0.0, y: 0.5 },
            padding: { left: 45, right: 15, top: 15, bottom: 15 },
            width: Fill,
            draw_bg: {
                color: #0000
                color_hover: #F2F4F733
                border_width: 0
            }
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 10},
                fn get_color(self) -> vec4 {
                    return #0008;
                }
            }
        }
    }
);

#[derive(Live, Widget, LiveHook)]
pub struct AgentButton {
    #[deref]
    view: View,

    #[rust]
    agent: Option<MaeAgent>,

    #[live(false)]
    create_new_chat: bool
}

impl Widget for AgentButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn set_text(&mut self, v: &str) {
        self.button(id!(button)).set_text(v);
    }
}

impl WidgetMatchEvent for AgentButton {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let Some(agent) = self.agent else { return };

        if self.button(id!(button)).clicked(actions) {
            if self.create_new_chat {
                store.chats.create_empty_chat_with_agent(agent);

                // Make sure text input is focused and other necessary setup happens.
                let widget_uid = self.widget_uid();
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    ChatHistoryCardAction::ChatSelected,
                );
            }

            cx.action(PromptInputAction::AgentSelected(agent))
        }
    }
}

impl AgentButton {
    pub fn set_agent(&mut self, agent: &MaeAgent) {
        self.set_text(&agent.name());
        self.chat_agent_avatar(id!(agent_avatar)).set_agent(agent);

        self.agent = Some(*agent);
    }
}

impl AgentButtonRef {
    pub fn set_agent(&mut self, agent: &MaeAgent) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_agent(agent);
        }
    }
}
