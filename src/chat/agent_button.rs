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
        flow: Right,
        width: Fill,
        height: 40,
        align: { x: 0.0, y: 0.5 },
        padding: { top: 4, top: 4 },
        spacing: 10,

        cursor: Hand
        show_bg: true,
        draw_bg: {
            color: #0000
        }

        agent_avatar = <ChatAgentAvatar> {
            padding: { left: 9 },
        }
        caption = <Label> {
            width: Fit,
            height: Fit,
            draw_text: {
                text_style: <BOLD_FONT>{font_size: 10},
                color: #000;
            }
        }
        description = <View> {
            visible: false,
            width: Fit,
            height: Fit,
            label = <Label> {
                width: Fit,
                height: Fit,
                draw_text: {
                    text_style: <REGULAR_FONT>{font_size: 9},
                    color: #667085,
                }
            }
        }

        animator: {
            hover = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.15}}
                    apply: {
                        draw_bg: {color: #F2F4F700}
                    }
                }
                on = {
                    from: {all: Snap}
                    apply: {
                        draw_bg: {color: #EAECEF88}
                    }
                }
            }
            down = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.5}}
                    ease: OutExp
                    apply: {
                        draw_bg: {down: 0.0}
                    }
                }
                on = {
                    ease: OutExp
                    from: {
                        all: Forward {duration: 0.2}
                    }
                    apply: {
                        draw_bg: {down: 1.0}
                    }
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
}

impl WidgetMatchEvent for AgentButton {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get_mut::<Store>().unwrap();

        let Some(agent) = self.agent else { return };

        if let Some(item) = actions.find_widget_action(self.view.widget_uid()) {
            if let ViewAction::FingerDown(fd) = item.cast() {
                if fd.tap_count == 1 {
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
    }
}

impl AgentButton {
    pub fn set_agent(&mut self, agent: &MaeAgent, show_description: bool) {
        self.label(id!(caption)).set_text(&agent.name());
        self.chat_agent_avatar(id!(agent_avatar)).set_agent(agent);

        self.view(id!(description)).set_visible(show_description);
        if show_description {
            self.view(id!(description.label)).set_text(&agent.short_description());
        }

        self.agent = Some(*agent);
    }
}

impl AgentButtonRef {
    pub fn set_agent(&mut self, agent: &MaeAgent, show_description: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_agent(agent, show_description);
        }
    }
}
