use makepad_widgets::*;
use moxin_mae::MaeBackend;

use crate::{chat::agent_button::AgentButtonWidgetRefExt, shared::computed_list::ComputedList};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::computed_list::*;
    import crate::chat::agent_button::*;

    AgentCard = <RoundedView> {
        width: Fill,
        height: 100,
        show_bg: false,
        draw_bg: {
            radius: 5,
            color: #F9FAFB,
        }
        button = <AgentButton> {
            width: Fill,
            height: Fill,

            draw_bg: {
                radius: 5,
            }
            agent_avatar = {
                image = {
                    width: 64,
                    height: 64,
                }
            }
            text_layout = {
                flow: Down,
            }
        }
    }

    AgentList = {{AgentList}} {
        width: Fill,
        height: Fit,
        flow: Down,
        spacing: 15,

        agent_row_template: <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            spacing: 15,

            first = <AgentCard> {}
            second = <AgentCard> {}
            third = <AgentCard> {}
        }
    }
}

#[derive(Live, Widget)]
pub struct AgentList {
    #[deref]
    deref: ComputedList,

    #[live]
    agent_row_template: Option<LivePtr>,
}

impl Widget for AgentList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }
}

impl LiveHook for AgentList {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        let agents = MaeBackend::available_agents();
        let agent_row_template = self.agent_row_template.clone();

        self.compute_from(agents.chunks(3), |chunk| {
            let row = WidgetRef::new_from_ptr(cx, agent_row_template);

            [id!(first), id!(second), id!(third)]
                .iter()
                .enumerate()
                .for_each(|(i, id)| {
                    if let Some(agent) = chunk.get(i) {
                        let cell = row.view(*id);
                        cell.apply_over(
                            cx,
                            live! {
                                show_bg: true,
                            },
                        );
                        cell.agent_button(id!(button)).set_agent(agent, true);
                    }
                });

            row
        });
    }
}
