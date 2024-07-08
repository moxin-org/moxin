use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import makepad_draw::shader::std::*;
    import makepad_draw::shader::draw_color::DrawColor;
    import crate::shared::widgets::*;
    import crate::shared::styles::*;

    ExternalLink = {{ExternalLink}} {
        width: Fit,
        height: Fit,
        flow: Down,
        link = <LinkLabel> {
            width: Fit,
            draw_text: {
                text_style: <REGULAR_FONT>{font_size: 9},
                fn get_color(self) -> vec4 {
                    return mix(
                        mix(
                            MODEL_LINK_FONT_COLOR,
                            MODEL_LINK_FONT_COLOR,
                            self.hover
                        ),
                        MODEL_LINK_FONT_COLOR,
                        self.pressed
                    )
                }
            }
        }
        underline = <Line> {
            width: Fill,
            height: 1,
            show_bg: true,
            draw_bg: {
                color: (MODEL_LINK_FONT_COLOR)
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ExternalLink {
    #[deref]
    view: View,

    #[rust]
    url: String,
}

impl Widget for ExternalLink {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ExternalLink {
    fn handle_actions(&mut self, _cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let link_label_clicked = self.link_label(id!(link)).clicked(actions);
        if link_label_clicked {
            self.open_url();
        }
    }
}

impl ExternalLink {
    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    fn open_url(&self) {
        let _ = robius_open::Uri::new(&self.url).open();
    }
}

impl ExternalLinkRef {
    pub fn set_url(&mut self, url: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_url(url);
        }
    }
}
