use makepad_widgets::*;
use moxin_mae::{MaeAgent, MaeBackend};

use crate::shared::{actions::ChatAction, computed_list::ComputedListWidgetExt};

use super::{
    agent_button::AgentButtonWidgetRefExt, model_selector_list::ModelSelectorAction,
    shared::ChatAgentAvatarWidgetExt,
};

#[derive(Debug, DefaultNone)]
pub enum PromptInputAction {
    AgentSelected(MaeAgent),
    None,
}

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::shared::computed_list::*;
    import crate::chat::agent_button::*;
    import crate::chat::shared::ChatAgentAvatar;

    ICON_PROMPT = dep("crate://self/resources/icons/prompt.svg")
    ICON_STOP = dep("crate://self/resources/icons/stop.svg")

    CircleButton = <MoxinButton> {
        padding: {right: 4},
        margin: {bottom: 2},

        draw_icon: {
            color: #fff
        }
        icon_walk: {width: 12, height: 12}
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

    PromptInput = {{PromptInput}} {
        flow: Overlay,
        height: Fit,
        agent_template: <View> {
            width: Fill,
            height: Fit,
            button = <AgentButton> {}
        }

        <View> {
            flow: Down,
            height: Fit,
            agent_autocomplete = <View> {
                height: Fit,
                visible: false,
                align: {x: 0.5, y: 1.0},
                margin: {bottom: 10},
                <RoundedView> {
                    flow: Down,
                    height: Fit,
                    padding: {bottom: 12.0, top: 12.0, right: 6.0, left: 6.0}
                    show_bg: true,
                    draw_bg: {
                        border_width: 1.0,
                        border_color: #D0D5DD,
                        color: #fff,
                        radius: 5.0
                    }

                    agent_search_input = <MoxinTextInput> {
                        width: Fill,
                        height: Fit,
                        margin: {bottom: 4},
                        empty_message: "Search for an agent",
                        draw_bg: {
                            radius: 5.0,
                            color: #fff
                        }
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 10},
                            color: #475467
                        }
                    }

                    list = <ComputedList> { height: Fit }
                }
            }

            <RoundedView> {
                flow: Down,
                width: Fill,
                height: Fit,
                padding: {top: 6, bottom: 6, left: 4, right: 10}
                spacing: 4,
                align: {x: 0.0, y: 1.0},

                show_bg: true,
                draw_bg: {
                    color: #fff,
                    radius: 10.0,
                    border_color: #D0D5DD,
                    border_width: 1.0,
                }

                selected_agent_bubble = <RoundedView> {
                    visible: false,
                    flow: Right,
                    width: Fit,
                    height: Fit,
                    align: {y: 0.5},
                    margin: 5.0
                    padding: {left: 10, right: 10, top: 8, bottom: 8}
                    show_bg: true,
                    draw_bg: {
                        color: #F2F4F7,
                        radius: 10.0,
                    }
                    agent_avatar = <ChatAgentAvatar> {
                        width: Fit,
                        height: Fit,
                        reasoner_avatar = {
                            image = {
                                width: 20, height: 20, margin: {right: 8}
                            }
                        }
                        research_scholar_avatar = {
                            image = {
                                width: 20, height: 20, margin: {right: 8}
                            }
                        }
                        search_assistant_avatar = {
                            image = {
                                width: 20, height: 20, margin: {right: 8}
                            }
                        }
                    }
                    <Label> {
                        text: "Chat with "
                        draw_text: {
                            text_style: <REGULAR_FONT>{font_size: 8},
                            color: #475467
                        }
                    }
                    selected_agent_label = <Label> {
                        margin: {right: 4},
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 8},
                            color: #000
                        }
                    }
                    agent_deselect_button = <MoxinButton> {
                        width: 8,
                        height: 8,
                        padding: 0,
                        draw_bg: {
                            color: #00000000,
                            color_hover: #00000000,
                            border_color_hover: #00000000,
                        }
                        icon_walk: {width: 8, height: 8}
                        draw_icon: {
                            svg_file: dep("crate://self/resources/icons/close.svg"),
                            color: #475467
                        }
                    }
                }

                <View> {
                    flow: Right,
                    width: Fill,
                    height: Fit,
                    prompt = <MoxinTextInput> {
                        width: Fill,
                        height: Fit,

                        empty_message: "Enter a message or @ an agent"
                        draw_bg: {
                            radius: 10.0
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
            }
        }

    }
}

#[derive(Widget, Live)]
pub struct PromptInput {
    #[deref]
    deref: View,

    #[live]
    agent_template: Option<LivePtr>,

    // see `was_at_added` function
    #[rust]
    prev_prompt: String,

    #[rust]
    agents_keyboard_focus_index: usize,

    #[rust]
    pub agent_selected: Option<MaeAgent>,
}

impl Widget for PromptInput {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.deref.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.deref.handle_event(cx, event, scope);

        if let Event::KeyUp(key_event) = event {
            if key_event.key_code == KeyCode::Escape {
                self.on_escape();
            }
        }

        // TODO: This should actually be "when the agent search input is focused"
        if self.view(id!(agent_autocomplete)).visible() {
            if let Event::KeyDown(key_event) = event {
                let delta = match key_event.key_code {
                    KeyCode::ArrowDown => 1,
                    KeyCode::ArrowUp => -1,
                    _ => 0,
                };

                if delta != 0 {
                    self.on_agent_search_keyboard_move(cx, delta);
                }
            }
        }

        self.widget_match_event(cx, event, scope);
    }
}

impl WidgetMatchEvent for PromptInput {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if let Some(current) = self.text_input(id!(prompt)).changed(actions) {
            self.on_prompt_changed(cx, current);
        }

        let agent_search_input = self.text_input(id!(agent_search_input));

        if let Some(current) = agent_search_input.changed(actions) {
            self.on_agent_search_changed(cx, current);
        }

        if let Some(current) = agent_search_input.returned(actions) {
            self.on_agent_search_submit(cx, current);
        }

        if self.button(id!(agent_deselect_button)).clicked(actions) {
            self.on_agent_deselected();
        }

        if let Some(action) = actions
            .iter()
            .find_map(|a| a.downcast_ref::<PromptInputAction>())
        {
            match action {
                PromptInputAction::AgentSelected(agent) => self.on_agent_selected(cx, agent),
                PromptInputAction::None => {}
            }
        }

        for action in actions.iter() {
            if let ModelSelectorAction::Selected(_) = action.as_widget_action().cast() {
                self.on_agent_deselected();
            }

            if let ChatAction::Start(_) = action.as_widget_action().cast() {
                self.on_agent_deselected();
            }
        }
    }
}

impl PromptInput {
    fn on_prompt_changed(&mut self, cx: &mut Cx, current: String) {
        if was_at_added(&self.prev_prompt, &current) {
            self.show_agent_autocomplete(cx);
        } else {
            self.hide_agent_autocomplete();
        }

        self.prev_prompt = current;
    }

    fn on_escape(&mut self) {
        let agent_autocomplete = self.view(id!(agent_autocomplete));
        agent_autocomplete.set_visible(false);
    }

    fn on_agent_selected(&mut self, cx: &mut Cx, agent: &MaeAgent) {
        self.agent_selected = Some(*agent);
        self.view(id!(agent_autocomplete)).set_visible(false);
        self.view(id!(selected_agent_bubble)).set_visible(true);

        self.chat_agent_avatar(id!(agent_avatar)).set_agent(agent);

        self.label(id!(selected_agent_label))
            .set_text(&agent.name());

        // TODO: Remove the inserted @

        self.text_input(id!(prompt)).set_key_focus(cx);
    }

    fn on_agent_deselected(&mut self) {
        self.agent_selected = None;
        self.view(id!(selected_agent_bubble)).set_visible(false);
    }

    fn on_agent_search_changed(&mut self, cx: &mut Cx, search: String) {
        // disallow multiline input
        self.text_input(id!(agent_search_input))
            .set_text(&search.replace("\n", " "));

        self.compute_agent_list(cx);
    }

    fn on_agent_search_submit(&mut self, cx: &mut Cx, current: String) {
        let agents = MaeBackend::available_agents();
        let agents = agents.iter();
        if let Some(agent) = filter_agents(agents, &current).nth(self.agents_keyboard_focus_index) {
            self.on_agent_selected(cx, agent);
        };
    }

    fn compute_agent_list(&mut self, cx: &mut Cx) {
        let search = self.text_input(id!(agent_search_input)).text();
        let list = self.computed_list(id!(agent_autocomplete.list));
        let agents = MaeBackend::available_agents();
        let agents = filter_agents(agents.iter(), &search);

        list.compute_from(agents.enumerate(), |(idx, agent)| {
            let widget = WidgetRef::new_from_ptr(cx, self.agent_template);

            let mut btn = widget.agent_button(id!(button));
            btn.set_agent(agent, true);

            if idx == self.agents_keyboard_focus_index {
                widget.apply_over(
                    cx,
                    live! {
                        show_bg: true,
                        draw_bg: {
                            color: #EAECEFff,
                        }
                    },
                );
            }

            widget
        });
    }

    fn show_agent_autocomplete(&mut self, cx: &mut Cx) {
        self.view(id!(agent_autocomplete)).set_visible(true);
        self.compute_agent_list(cx);
        self.text_input(id!(agent_search_input)).set_key_focus(cx);
    }

    fn hide_agent_autocomplete(&mut self) {
        self.view(id!(agent_autocomplete)).set_visible(false);
        self.text_input(id!(agent_search_input)).set_text("");
        self.agents_keyboard_focus_index = 0;
    }

    fn on_agent_search_keyboard_move(&mut self, cx: &mut Cx, delta: i32) {
        let items_len = self.computed_list(id!(agent_autocomplete.list)).len();

        if items_len == 0 {
            return;
        }

        self.agents_keyboard_focus_index = self
            .agents_keyboard_focus_index
            .saturating_add_signed(delta as isize)
            .clamp(0, items_len - 1);

        self.compute_agent_list(cx);
    }
}

impl LiveHook for PromptInput {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.compute_agent_list(cx);
    }
}

impl PromptInputRef {
    pub fn reset_text(&mut self, cx: &mut Cx, set_key_focus: bool) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };

        let prompt_input = inner.text_input(id!(prompt));
        prompt_input.set_text("");
        prompt_input.set_cursor(0, 0);

        inner.hide_agent_autocomplete();
        inner.prev_prompt.clear();

        if set_key_focus {
            prompt_input.set_key_focus(cx);
        }
    }
}

// workaround to detect if '@' was added to the prompt
// this doesn't take into account mouse cursor position so it can give false positives
// when copy-pasting text.
fn was_at_added(prev: &str, current: &str) -> bool {
    let char_added = current.len() == prev.len() + 1;
    let at_added = current.chars().filter(|c| *c == '@').count()
        == prev.chars().filter(|c| *c == '@').count() + 1;

    char_added && at_added
}

fn filter_agents<'a, A: Iterator<Item = &'a MaeAgent>>(
    agents: A,
    search: &str,
) -> impl Iterator<Item = &'a MaeAgent> {
    let terms = search
        .split_whitespace()
        .map(|s| s.to_ascii_lowercase())
        .collect::<Vec<_>>();

    agents.filter(move |agent| {
        terms
            .iter()
            .all(|term| agent.name().to_ascii_lowercase().contains(term))
    })
}
