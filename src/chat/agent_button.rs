use makepad_widgets::*;
use moxin_mae::MaeAgent;

use super::prompt_input::PromptInputAction;

live_design!(
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import crate::shared::styles::*;
    import crate::shared::widgets::*;

    AgentButton = {{AgentButton}} {
        flow: Overlay,
        align: { x: 0.0, y: 0.5 },
        <View> {
            width: Fit,
            height: Fit,
            padding: { left: 9 },
            <Image> {
                width: 24,
                height: 24,
                source: dep("crate://self/resources/images/agent.png")
            }
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
    pub fn set_agent(&mut self, agent: MaeAgent) {
        self.set_text(&agent.name());
        self.agent = Some(agent);
    }
}

impl AgentButtonRef {
    pub fn set_agent(&mut self, agent: MaeAgent) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_agent(agent);
        }
    }
}
