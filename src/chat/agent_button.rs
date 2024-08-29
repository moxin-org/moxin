use makepad_widgets::*;
use moxin_mae::MaeAgent;

use super::prompt_input::PromptInputAction;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    REASONER_AGENT_ICON = dep("crate://self/resources/images/reasoner_agent_icon.png")
    RESEARCH_SCHOLAR_ICON = dep("crate://self/resources/images/research_scholar_agent_icon.png")
    SEARCH_ASSISTANT_ICON = dep("crate://self/resources/images/search_assistant_agent_icon.png")

    Avatar = <View> {
        width: Fit,
        height: Fit,
        visible: false
        padding: { left: 9 },
        image = <Image> {
            width: 24,
            height: 24,
        }
    }

    AgentButton = {{AgentButton}} {
        flow: Overlay,
        align: { x: 0.0, y: 0.5 },
        reasoner_avatar = <Avatar> {
            visible: true
            image = { source: (REASONER_AGENT_ICON) }
        }
        research_scholar_avatar = <Avatar> {
            image = { source: (RESEARCH_SCHOLAR_ICON) }
        }
        search_assistant_avatar = <Avatar> {
            image = { source: (SEARCH_ASSISTANT_ICON) }
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
}

impl Widget for AgentButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if let Event::Actions(actions) = event {
            if self.button(id!(button)).clicked(actions) {
                cx.action(PromptInputAction::AgentSelected(self.agent.unwrap()))
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn set_text(&mut self, v: &str) {
        self.button(id!(button)).set_text(v);
    }
}

impl AgentButton {
    pub fn set_agent(&mut self, cx: &mut Cx, agent: MaeAgent) {
        self.set_text(&agent.name());

        self.view(id!(reasoner_avatar)).set_visible(false);
        self.view(id!(research_scholar_avatar)).set_visible(false);
        self.view(id!(search_assistant_avatar)).set_visible(false);
        match agent {
            MaeAgent::Reasoner => {
                self.view(id!(reasoner_avatar)).set_visible(true);
            },
            MaeAgent::ResearchScholar => {
                self.view(id!(research_scholar_avatar)).set_visible(true);
            }
            MaeAgent::SearchAssistant => {
                self.view(id!(search_assistant_avatar)).set_visible(true);
            }
        };

        self.agent = Some(agent);
    }
}

impl AgentButtonRef {
    pub fn set_agent(&mut self, cx: &mut Cx, agent: MaeAgent) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_agent(cx, agent);
        }
    }
}
