use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;
    import makepad_draw::shader::draw_color::DrawColor;
    import crate::shared::widgets::*;
    import crate::shared::styles::*;
    import crate::shared::modal::*;

    Tooltip = {{Tooltip}}{   
        width: Fill,
        height: Fill,

        flow: Overlay
        align: {x: 0.0, y: 0.0}

        draw_bg: {
            fn pixel(self) -> vec4 {
                return vec4(0., 0., 0., 0.0)
            }
        }

        content: <View> {
            flow: Overlay
            width: Fit
            height: Fit

            <RoundedView> {
                width: Fit,
                height: Fit,
    
                padding: 16,
    
                draw_bg: {
                    color: #fff,
                    border_width: 1.0,
                    border_color: #D0D5DD,
                    radius: 2.
                }
    
                tooltip_label = <Label> {
                    width: 270,
                    draw_text: {
                        text_style: <REGULAR_FONT>{font_size: 9},
                        text_wrap: Word,
                        color: #000
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct Tooltip {
    #[rust]
    opened: bool,

    #[live]
    #[find]
    content: View,

    #[redraw]
    #[rust(DrawList2d::new(cx))]
    draw_list: DrawList2d,

    #[live]
    draw_bg: DrawQuad,
    #[layout]
    layout: Layout,
    #[walk]
    walk: Walk,
}

impl Widget for Tooltip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.content.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, _walk: Walk) -> DrawStep {
        self.draw_list.begin_overlay_reuse(cx);

        cx.begin_pass_sized_turtle(self.layout);
        self.draw_bg.begin(cx, self.walk, self.layout);

        if self.opened {
            let _ = self.content.draw_all(cx, scope);
        }

        self.draw_bg.end(cx);

        cx.end_pass_sized_turtle();
        self.draw_list.end(cx);

        DrawStep::done()
    }
}

impl TooltipRef {
    pub fn set_text_and_pos(&mut self, cx: &mut Cx, text: &str, pos: DVec2) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.apply_over(
                cx,
                live! {
                    content: { margin: { left: (pos.x), top: (pos.y) } }
                },
            );
            inner.label(id!(tooltip_label)).set_text(text);
            inner.redraw(cx);
        }
    }

    pub fn show_tooltip(&mut self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.opened = true;
        }
    }

    pub fn hide_tooltip(&mut self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.opened = false;
        }
    }
}
