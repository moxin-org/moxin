use makepad_widgets::*;
use moxin_mae::MaeAgent;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    REASONER_AGENT_ICON = dep("crate://self/resources/images/reasoner_agent_icon.png")
    RESEARCH_SCHOLAR_ICON = dep("crate://self/resources/images/research_scholar_agent_icon.png")
    SEARCH_ASSISTANT_ICON = dep("crate://self/resources/images/search_assistant_agent_icon.png")

    ChatModelAvatar = <RoundedView> {
        width: 24,
        height: 24,

        show_bg: true,
        draw_bg: {
            color: #444D9A,
            radius: 6,
        }

        align: {x: 0.5, y: 0.5},

        avatar_label = <Label> {
            width: Fit,
            height: Fit,
            draw_text:{
                text_style: <BOLD_FONT>{font_size: 10},
                color: #fff,
            }
            text: "P"
        }
    }

    ChatAgentAvatar = {{ChatAgentAvatar}} {
        width: Fit,
        height: Fit,
        reasoner_avatar = <View> {
            width: Fit, height: Fit, visible: true
            image = <Image> { width: 24, height: 24, source: (REASONER_AGENT_ICON) }
        }
        research_scholar_avatar = <View> {
            width: Fit, height: Fit
            image = <Image> { width: 24, height: 24, source: (RESEARCH_SCHOLAR_ICON) }
        }
        search_assistant_avatar = <View> {
            width: Fit, height: Fit
            image = <Image> { width: 24, height: 24, source: (SEARCH_ASSISTANT_ICON) }
        }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct ChatAgentAvatar {
    #[deref]
    view: View,
}

impl Widget for ChatAgentAvatar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatAgentAvatar {
    pub fn set_agent(&mut self, agent: &MaeAgent) {
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
    }
}

impl ChatAgentAvatarRef {
    pub fn set_agent(&mut self, agent: &MaeAgent) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_agent(agent);
        }
    }

    pub fn set_visible(&mut self, visible: bool) -> () {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.visible = visible;
        }
    }
}

