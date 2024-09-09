use makepad_widgets::*;
use moxin_mae::MaeAgent;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

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
        reasoner_agent_icon: dep("crate://self/resources/images/reasoner_agent_icon.png")
        research_scholar_icon: dep("crate://self/resources/images/research_scholar_agent_icon.png")
        search_assistant_icon: dep("crate://self/resources/images/search_assistant_agent_icon.png")
        width: Fit,
        height: Fit,
        image = <Image> { width: 24, height: 24 }
    }
}

#[derive(Live, Widget, LiveHook)]
pub struct ChatAgentAvatar {
    #[deref]
    view: View,

    #[live]
    reasoner_agent_icon: LiveValue,

    #[live]
    research_scholar_icon: LiveValue,

    #[live]
    search_assistant_icon: LiveValue,

    // To avoid requesting `cx` on `set_agent`, which would cause a lot of changes in chain.
    #[rust]
    pending_image_update: Option<LiveValue>,
}

impl Widget for ChatAgentAvatar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if let Some(dep) = self.pending_image_update.take() {
            self.apply_over(
                cx,
                live! {
                    image = {
                        source: (dep)
                    }
                },
            )
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl ChatAgentAvatar {
    pub fn set_agent(&mut self, agent: &MaeAgent) {
        let dep = match agent {
            MaeAgent::Reasoner => self.reasoner_agent_icon.clone(),
            MaeAgent::ResearchScholar => self.research_scholar_icon.clone(),
            MaeAgent::SearchAssistant => self.search_assistant_icon.clone(),
        };

        self.pending_image_update = Some(dep);
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
