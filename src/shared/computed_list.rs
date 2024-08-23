use makepad_widgets::*;

live_design!(
    ComputedList = {{ComputedList}} {
        flow: Down,
        width: Fill,
        height: Fill,
    }
);

/// Minimalistic list of widgets mapped from your data, eagerly rendered and
/// with a known size.
#[derive(Live, Widget, LiveHook)]
pub struct ComputedList {
    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[redraw]
    #[rust]
    area: Area,

    #[rust]
    items: Vec<WidgetRef>,
}

impl Widget for ComputedList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.items.iter().for_each(|item| {
            item.handle_event(cx, event, scope);
        });
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        self.items.iter().for_each(|item| {
            item.draw_all(cx, scope);
        });
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl ComputedList {
    /// Build each widget from mapped from your data.
    pub fn compute_from<T, I: Iterator<Item = T>>(
        &mut self,
        iter: I,
        f: impl FnMut(T) -> WidgetRef,
    ) {
        self.items = iter.map(f).collect();
    }

    /// Iterate over the current items/widgets.
    pub fn items(&self) -> impl Iterator<Item = &WidgetRef> {
        self.items.iter()
    }
}

impl ComputedListRef {
    /// Calls `compute_from` on the inner widget.
    pub fn compute_from<T, I: Iterator<Item = T>>(&self, iter: I, f: impl FnMut(T) -> WidgetRef) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.compute_from(iter, f);
        }
    }
}
