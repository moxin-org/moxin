use makepad_widgets::*;

use crate::{
    data::{chats::chat::ChatID, store::Store},
    shared::actions::TooltipAction,
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::FadeView;
    import crate::shared::widgets::MoxinButton;
    import crate::shared::widgets::MoxinSlider;
    import crate::shared::widgets::MoxinSwitch;
    import crate::shared::widgets::MoxinTextInput;
    import makepad_draw::shader::std::*;

    ICON_CLOSE_PANEL = dep("crate://self/resources/icons/close_right_panel.svg")
    ICON_OPEN_PANEL = dep("crate://self/resources/icons/open_right_panel.svg")

    ChatParamsActions = <View> {
        height: Fit
        flow: Right

        <View> {
            width: Fill
            height: Fit
        }


        close_panel_button = <MoxinButton> {
            width: Fit,
            height: Fit,
            icon_walk: {width: 20, height: 20},
            draw_icon: {
                svg_file: (ICON_CLOSE_PANEL),
                fn get_color(self) -> vec4 {
                    return #475467;
                }
            }
        }

        open_panel_button = <MoxinButton> {
            width: Fit,
            height: Fit,
            visible: false,
            icon_walk: {width: 20, height: 20},
            draw_icon: {
                svg_file: (ICON_OPEN_PANEL),
                fn get_color(self) -> vec4 {
                    return #475467;
                }
            }
        }
    }

    ChatParams = {{ChatParams}} {
        flow: Overlay,
        width: Fit,
        height: Fill,

        main_content = <FadeView> {
            width: 300
            height: Fill
            <View> {
                width: Fill
                height: Fill
                padding: {top: 70, left: 25.0, right: 25.0}
                spacing: 35
                flow: Down
                show_bg: true
                draw_bg: {
                    color: #F2F4F7
                }

                label = <Label> {
                    draw_text: {
                        text_style: <BOLD_FONT>{font_size: 12}
                        color: #000
                    }
                    text: "Inference Parameters"
                }

                <View> {
                    flow: Down
                    spacing: 24

                    temperature = <MoxinSlider> {
                        default: 1.0
                        text: "Temperature"
                        min: 0.0
                        max: 2.0
                    }

                    top_p = <MoxinSlider> {
                        text: "Top P"
                        min: 0.0
                        max: 1.0
                    }

                    <View> {
                        flow: Right
                        height: Fit
                        width: Fill
                        align: {y: 0.5}
                        padding: {left: 4}
                        stream_label = <Label> {
                            width: Fill
                            draw_text: {
                                text_style: <BOLD_FONT>{font_size: 10},
                                color: #000
                            }
                            text: "Stream"
                        }
                        stream = <MoxinSwitch> {
                            // Match the default value to avoid the animation on start.
                            animator: {
                                selected = {
                                    default: on
                                }
                            }
                        }
                    }

                    max_tokens = <MoxinSlider> {
                        text: "Max Tokens"
                        min: 100.0
                        max: 2048.0
                        step: 1.0
                    }

                    <View> {
                        flow: Down
                        height: Fit
                        width: Fill
                        spacing: 12
                        padding: {left: 4}
                        stop_label = <Label> {
                            width: Fill,
                            draw_text: {
                                text_style: <BOLD_FONT>{font_size: 10},
                                color: #000
                            }
                            text: "Stop"
                        }
                        stop = <MoxinTextInput> {
                            width: Fill,
                            // TODO: This should be something like min-height, allowing
                            // the text input to grow.
                            height: 65,
                            empty_message: ""
                            draw_bg: {
                                radius: 5.0
                                color: #fff
                                border_width: 1.0,
                                border_color: #D9D9D9,
                            }
                            draw_text: {
                                text_style: {font_size: 10},
                            }
                        }
                    }

                    frequency_penalty = <MoxinSlider> {
                        text: "Frequency Penalty"
                        min: 0.0
                        max: 1.0
                    }

                    presence_penalty = <MoxinSlider> {
                        text: "Presence Penalty"
                        min: 0.0
                        max: 1.0
                    }
                }
            }
        }

        <ChatParamsActions> {
            padding: {top: 58, left: 25, right: 25}
        }

        animator: {
            panel = {
                default: show,
                show = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {main_content = { width: 300, draw_bg: {opacity: 1.0} }}
                }
                hide = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {main_content = { width: 110, draw_bg: {opacity: 0.0} }}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ChatParams {
    #[deref]
    view: View,

    #[animator]
    animator: Animator,

    #[rust]
    current_chat_id: Option<ChatID>,
}

impl Widget for ChatParams {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();

        if let Some(chat) = store.chats.get_current_chat() {
            self.visible = true;

            let chat = chat.borrow();
            let ip = &chat.inferences_params;

            let temperature = self.slider(id!(temperature));
            let top_p = self.slider(id!(top_p));
            let max_tokens = self.slider(id!(max_tokens));
            let frequency_penalty = self.slider(id!(frequency_penalty));
            let presence_penalty = self.slider(id!(presence_penalty));
            let stop = self.text_input(id!(stop));
            let stream = self.check_box(id!(stream));

            temperature.set_value(ip.temperature.into());
            top_p.set_value(ip.top_p.into());
            max_tokens.set_value(ip.max_tokens.into());
            frequency_penalty.set_value(ip.frequency_penalty.into());
            presence_penalty.set_value(ip.presence_penalty.into());
            stop.set_text(&ip.stop);

            // Currently, `selected` and `set_selected` interact with the animator of
            // the widget to do what they do. To avoid some visual issues, we should not
            // trigger the animator unnecessarily. This is a workaround.
            if stream.selected(cx) != ip.stream {
                stream.set_selected(cx, ip.stream);
            }
        } else {
            self.visible = false;
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ChatParams {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        self.process_tooltip_actions(cx, actions, scope);

        let store = scope.data.get_mut::<Store>().unwrap();
        let close = self.button(id!(close_panel_button));
        let open = self.button(id!(open_panel_button));

        if close.clicked(&actions) {
            close.set_visible(false);
            open.set_visible(true);
            self.animator_play(cx, id!(panel.hide));
        }

        if open.clicked(&actions) {
            open.set_visible(false);
            close.set_visible(true);
            self.animator_play(cx, id!(panel.show));
        }

        if let Some(chat) = store.chats.get_current_chat() {
            let mut chat = chat.borrow_mut();

            if self.current_chat_id != Some(chat.id) {
                self.current_chat_id = Some(chat.id);
                self.redraw(cx);
            }

            let ip = &mut chat.inferences_params;

            if let Some(value) = self.slider(id!(temperature)).slided(&actions) {
                ip.temperature = value as f32;
            }

            if let Some(value) = self.slider(id!(top_p)).slided(&actions) {
                ip.top_p = value as f32;
            }

            if let Some(value) = self.slider(id!(max_tokens)).slided(&actions) {
                ip.max_tokens = value as u32;
            }

            if let Some(value) = self.slider(id!(frequency_penalty)).slided(&actions) {
                ip.frequency_penalty = value as f32;
            }

            if let Some(value) = self.slider(id!(presence_penalty)).slided(&actions) {
                ip.presence_penalty = value as f32;
            }

            if let Some(value) = self.text_input(id!(stop)).changed(&actions) {
                ip.stop = value;
            }

            if let Some(value) = self.check_box(id!(stream)).changed(actions) {
                ip.stream = value;
            }
        }
    }
}

impl ChatParams {
    fn process_tooltip_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        if self.animator.animator_in_state(cx, id!(panel.hide)) {
            return;
        }

        self.process_tooltip_actions_for_slider(
            id!(temperature),
            "Influences the randomness of the model’s output. A higher value leads to more random and diverse responses, while a lower value produces more predictable outputs.".to_string(),
            cx, actions, scope
        );

        self.process_tooltip_actions_for_slider(
            id!(top_p),
            "Top P, also known as nucleus sampling, is another parameter that influences the randomness of LLM output. This parameter determines the threshold probability for including tokens in a candidate set used by the LLM to generate output. Lower values of this parameter result in more precise and fact-based responses from the LLM, while higher values increase randomness and diversity in the generated output.".to_string(),
            cx, actions, scope
        );

        self.process_tooltip_actions_for_label(
            id!(stream_label),
            "Streaming is the sending of words as they are created by the AI language model one at a time, so you can show them as they are being generated.".to_string(),
            cx, actions, scope
        );

        self.process_tooltip_actions_for_slider(
            id!(max_tokens),
            "The max tokens parameter sets the upper limit for the total number of tokens, encompassing both the input provided to the LLM as a prompt and the output tokens generated by the LLM in response to that prompt.".to_string(),
            cx, actions, scope
        );

        self.process_tooltip_actions_for_slider(
            id!(frequency_penalty),
            "This parameter is used to discourage the model from repeating the same words or phrases too frequently within the generated text. It is a value that is added to the log-probability of a token each time it occurs in the generated text. A higher frequency_penalty value will result in the model being more conservative in its use of repeated tokens.".to_string(),
            cx, actions, scope
        );

        self.process_tooltip_actions_for_slider(
            id!(presence_penalty),
            "This parameter is used to encourage the model to include a diverse range of tokens in the generated text. It is a value that is subtracted from the log-probability of a token each time it is generated. A higher presence_penalty value will result in the model being more likely to generate tokens that have not yet been included in the generated text.".to_string(),
            cx, actions, scope
        );

        self.process_tooltip_actions_for_label(
            id!(stop_label),
            "Stop sequences are used to make the model stop generating tokens at a desired point, such as the end of a sentence or a list. The model response will not contain the stop sequence and you can pass up to four stop sequences.".to_string(),
            cx, actions, scope
        );
    }

    fn process_tooltip_actions_for_slider(
        &mut self,
        slider_id: &[LiveId],
        text: String,
        cx: &mut Cx,
        actions: &Actions,
        scope: &mut Scope,
    ) {
        let widget_uid = self.widget_uid();
        let slider = self.slider(slider_id);
        if let Some(rect) = slider.label_hover_in(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                TooltipAction::Show(text, rect.pos.x - 300.0, rect.pos.y - 30.0),
            );
        }
        if slider.label_hover_out(&actions) {
            cx.widget_action(widget_uid, &scope.path, TooltipAction::Hide);
        }
    }

    fn process_tooltip_actions_for_label(
        &mut self,
        label_id: &[LiveId],
        text: String,
        cx: &mut Cx,
        actions: &Actions,
        scope: &mut Scope,
    ) {
        let widget_uid = self.widget_uid();
        let label = self.label(label_id);
        if let Some(rect) = label.hover_in(&actions) {
            cx.widget_action(
                widget_uid,
                &scope.path,
                TooltipAction::Show(text, rect.pos.x - 300.0, rect.pos.y - 30.0),
            );
        }
        if label.hover_out(&actions) {
            cx.widget_action(widget_uid, &scope.path, TooltipAction::Hide);
        }
    }
}
