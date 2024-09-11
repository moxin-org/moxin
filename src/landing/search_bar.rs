use crate::data::store::StoreAction;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import makepad_draw::shader::std::*;

    import crate::landing::sorting::Sorting;

    ICON_SEARCH = dep("crate://self/resources/icons/search.svg")

    SearchBar = {{SearchBar}} {
        flow: Right,
        width: Fill,
        height: 100,
        spacing: 80,
        padding: {left: 20},
        align: {x: 0.0, y: 0.5},

        show_bg: true,

        draw_bg: {
            color: #EBFCFF,
            instance color2: #CBE3E8,
            fn get_color(self) -> vec4 {
                let coef = self.rect_size.y / self.rect_size.x;

                let distance_vec = self.pos - vec2(0.8, 1.1);
                let norm_distance = length(vec2(distance_vec.x, distance_vec.y * coef) * 2.2);

                if pow(norm_distance, 1.4) > 1.0 {
                    return self.color;
                } else {
                    return mix(self.color2, self.color, pow(norm_distance, 1.4));
                }
            }

            fn pixel(self) -> vec4 {
                return Pal::premul(self.get_color());
            }
        }

        input_container = <RoundedView> {
            width: Fill,
            height: Fit,

            show_bg: true,
            draw_bg: {
                color: #fff
            }

            padding: {top: 3, bottom: 3, left: 20, right: 20}
            margin: {left: 30, right: 30}

            spacing: 4,
            align: {x: 0.0, y: 0.5},

            draw_bg: {
                radius: 9.0,
                border_color: #D0D5DD,
                border_width: 1.0,
            }

            <Icon> {
                draw_icon: {
                    svg_file: (ICON_SEARCH),
                    fn get_color(self) -> vec4 {
                        return #666;
                    }
                }
                icon_walk: {width: 17, height: 17}
            }

            input = <MoxinTextInput> {
                width: Fill,
                height: Fit,
                empty_message: "Search Model by Keyword"
            }
        }

        search_sorting = <View> {
            width: 300,
            height: Fit,
            margin: {left: 30, right: 30},
            <Sorting> {}
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SearchBar {
    #[deref]
    view: View,

    #[rust]
    search_timer: Timer,

    #[live(0.3)]
    search_debounce_time: f64,
}

impl Widget for SearchBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if self.search_timer.is_event(event).is_some() {
            self.search_timer = Timer::default();

            let input = self.text_input(id!(input));
            let keywords = input.text();
            const MIN_SEARCH_LENGTH: usize = 2;

            if keywords.len() > MIN_SEARCH_LENGTH {
                let widget_uid = self.widget_uid();
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    StoreAction::Search(keywords.to_string()),
                );
            } else if keywords.len() == 0 {
                let widget_uid = self.widget_uid();
                cx.widget_action(widget_uid, &scope.path, StoreAction::ResetSearch);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SearchBar {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let input = self.text_input(id!(input));

        if let Some(keywords) = input.returned(actions) {
            if keywords.len() > 0 {
                let widget_uid = self.widget_uid();
                cx.widget_action(
                    widget_uid,
                    &scope.path,
                    StoreAction::Search(keywords.to_string()),
                );
            } else {
                let widget_uid = self.widget_uid();
                cx.widget_action(widget_uid, &scope.path, StoreAction::ResetSearch);
            }
        }

        if let Some(_) = input.changed(actions) {
            cx.stop_timer(self.search_timer);
            self.search_timer = cx.start_timeout(self.search_debounce_time);
        }
    }
}
