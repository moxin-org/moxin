use makepad_widgets::*;

use super::portal::PortalAction;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::portal::*;

    Modal = {{Modal}} {
        width: Fill
        height: Fill
        flow: Overlay
        align: {x: 0.5, y: 0.5}

        draw_bg: {
            fn pixel(self) -> vec4 {
                return vec4(0., 0., 0., 0.0)
            }
        }

        bg_view: <View> {
            width: Fill
            height: Fill
            show_bg: true
            draw_bg: {
                fn pixel(self) -> vec4 {
                    return vec4(0., 0., 0., 0.7)
                }
            }
        }

        content: <View> {
            flow: Overlay
            width: Fit
            height: Fit
        }
    }
}

#[derive(Live, Widget)]
pub struct Modal {
    #[live]
    #[find]
    content: View,
    #[live]
    bg_view: View,

    #[redraw]
    #[rust(DrawList2d::new(cx))]
    draw_list: DrawList2d,

    #[live]
    draw_bg: DrawQuad,
    #[layout]
    layout: Layout,
    #[walk]
    walk: Walk,

    #[rust]
    opened: bool,
}

impl LiveHook for Modal {
    fn after_apply(&mut self, cx: &mut Cx, _apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        self.draw_list.redraw(cx);
    }
}

impl Widget for Modal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.opened {
            return;
        }

        // When passing down events we need to suspend the sweep lock
        // because regular View instances won't respond to events if the sweep lock is active.
        cx.sweep_unlock(self.draw_bg.area());
        self.content.handle_event(cx, event, scope);
        cx.sweep_lock(self.draw_bg.area());

        self.widget_match_event(cx, event, scope);

        // Check if there was a click outside of the content (bg), then close if true.
        let content_rec = self.content.area().rect(cx);
        if let Hit::FingerUp(fe) = event.hits_with_sweep_area(cx, self.draw_bg.area(), self.draw_bg.area()) {
            if !content_rec.contains(fe.abs) {
                let widget_uid = self.content.widget_uid();
                cx.widget_action(widget_uid, &scope.path, PortalAction::Close);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_list.begin_overlay_reuse(cx);

        cx.begin_pass_sized_turtle(self.layout);
        self.draw_bg.begin(cx, self.walk, self.layout);

        if self.opened {
            let _ = self
                .bg_view
                .draw_walk(cx, scope, walk.with_abs_pos(DVec2 { x: 0., y: 0. }));
            let _ = self.content.draw_all(cx, scope);
        }

        self.draw_bg.end(cx);

        cx.end_pass_sized_turtle();
        self.draw_list.end(cx);

        DrawStep::done()
    }
}

impl WidgetMatchEvent for Modal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for action in actions {
            match action.as_widget_action().cast::<PortalAction>() {
                PortalAction::Close => {
                    self.opened = false;
                    self.draw_bg.redraw(cx);
                    cx.sweep_unlock(self.draw_bg.area())
                }
                _ => {}
            }
        }
    }
}

impl Modal {
    pub fn open_modal(&mut self, cx: &mut Cx) {
        self.opened = true;
        self.redraw(cx);
        cx.sweep_lock(self.draw_bg.area());
    }

    pub fn close_modal(&mut self, cx: &mut Cx) {
        self.opened = false;
        self.draw_bg.redraw(cx);
        cx.sweep_unlock(self.draw_bg.area())
    }
}

impl ModalRef {
    pub fn open_modal(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.open_modal(cx);
        }
    }

    pub fn close_modal(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close_modal(cx);
        }
    }
}
