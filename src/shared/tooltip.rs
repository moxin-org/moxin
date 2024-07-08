use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;
    import makepad_draw::shader::draw_color::DrawColor;
    import crate::shared::widgets::*;
    import crate::shared::styles::*;

    Tooltip = {{Tooltip}} {
        width: Fill,
        height: Fill,
        flow: Down,
        
        <RoundedView> {
            width: Fit,
            height: Fit,

            padding: 10,

            draw_bg: {
                color: #fff,
                border_width: 1.0,
                border_color: #D0D5DD,
                radius: 2.
            }

            label = <Label> {
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

#[derive(Live, LiveHook, Widget)]
pub struct Tooltip {
    #[deref]
    view: View,
}

impl Widget for Tooltip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl TooltipRef {
    pub fn set_text(&mut self, s:&str){
        self.label(id!(label)).set_text(s);
    }
}
