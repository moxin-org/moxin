use crate::data::store::{Store, StoreAction};
use crate::landing::model_list::ModelListAction;
use crate::landing::search_bar::SearchBarWidgetExt;
use crate::landing::sorting::SortingWidgetExt;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::VerticalFiller;
    import crate::landing::search_bar::SearchBar;
    import crate::landing::model_list::ModelList;
    import crate::landing::sorting::Sorting;
    import crate::landing::downloads::Downloads;
    import crate::landing::agent_list::AgentList;

    Heading = <View> {
        width: Fill,
        height: Fit,
        spacing: 30,

        align: {x: 0.5, y: 0.5},

        heading_no_filters = <View> {
            width: Fit,
            height: 50,

            align: {x: 0.5, y: 0.5},

            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 16},
                    color: #000
                }
                text: "Explore"
            }
        }

        heading_with_filters = <View> {
            width: Fit,
            height: 50,

            align: {x: 0.5, y: 0.5},

            results = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 16},
                    color: #000
                }
                text: "12 Results"
            }
            keyword = <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 16},
                    color: #000
                }
                text: " for \"Open Hermes\""
            }
        }

        <VerticalFiller> {}

        sorting = <Sorting> {
            width: Fit,
            height: Fit,
            visible: false
        }
    }

    LandingScreen = {{LandingScreen}} {
        width: Fill,
        height: Fill,
        flow: Overlay,

        <View> {
            width: Fill,
            height: Fill,
            flow: Down,

            search_bar = <SearchBar> {}

            <View> {
                width: Fill,
                height: Fit,
                spacing: 30,
                margin: { left: 50, right: 50, top: 30 },

                <AgentList> {}
            }

            models = <View> {
                width: Fill,
                height: Fill,
                flow: Down,
                spacing: 30,
                margin: { left: 50, right: 50, top: 30 },

                <Heading> {}
                <ModelList> {}
            }
            downloads = <Downloads> {}
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum SearchBarState {
    #[default]
    ExpandedWithoutFilters,
    ExpandedWithFilters,
    CollapsedWithoutFilters,
    CollapsedWithFilters,
}

#[derive(Live, LiveHook, Widget)]
pub struct LandingScreen {
    #[deref]
    view: View,

    #[rust]
    search_bar_state: SearchBarState,
}

impl Widget for LandingScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let search = &scope.data.get::<Store>().unwrap().search;
        if search.is_pending() || search.was_error() {
            self.view(id!(heading_with_filters)).set_visible(false);
            self.view(id!(heading_no_filters)).set_visible(false);
            self.sorting(id!(sorting)).set_visible(cx, false);
        } else if let Some(keyword) = search.keyword.clone() {
            self.view(id!(heading_with_filters)).set_visible(true);
            self.view(id!(heading_no_filters)).set_visible(false);

            let models = &search.models;
            let models_count = models.len();
            self.label(id!(heading_with_filters.results))
                .set_text(&format!("{} Results", models_count));
            self.label(id!(heading_with_filters.keyword))
                .set_text(&format!(" for \"{}\"", keyword));
        } else {
            self.view(id!(heading_with_filters)).set_visible(false);
            self.view(id!(heading_no_filters)).set_visible(true);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for LandingScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        for action in actions.iter() {
            match action.as_widget_action().cast() {
                ModelListAction::ScrolledAtTop => {
                    if self.search_bar_state == SearchBarState::CollapsedWithoutFilters {
                        self.search_bar_state = SearchBarState::ExpandedWithoutFilters;
                        self.search_bar(id!(search_bar)).expand(cx);
                        self.sorting(id!(sorting)).set_visible(cx, false);
                        self.redraw(cx);
                    }
                }
                ModelListAction::ScrolledNotAtTop => {
                    let collapse: bool;
                    match self.search_bar_state {
                        SearchBarState::ExpandedWithoutFilters => {
                            self.search_bar_state = SearchBarState::CollapsedWithoutFilters;
                            collapse = true;
                        }
                        SearchBarState::ExpandedWithFilters => {
                            self.search_bar_state = SearchBarState::CollapsedWithFilters;
                            collapse = true;
                        }
                        _ => {
                            collapse = false;
                        }
                    }

                    if collapse {
                        let search = &scope.data.get::<Store>().unwrap().search;
                        self.search_bar(id!(search_bar))
                            .collapse(cx, search.sorted_by);
                        self.sorting(id!(sorting)).set_visible(cx, false);
                        self.redraw(cx);
                    }
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                StoreAction::Search(_keywords) => match self.search_bar_state {
                    SearchBarState::CollapsedWithoutFilters => {
                        self.search_bar_state = SearchBarState::CollapsedWithFilters;
                    }
                    SearchBarState::ExpandedWithoutFilters => {
                        self.search_bar_state = SearchBarState::ExpandedWithFilters;

                        let search = &scope.data.get::<Store>().unwrap().search;
                        let sorting_ref = self.sorting(id!(sorting));
                        sorting_ref.set_visible(cx, true);
                        sorting_ref.set_selected_item(search.sorted_by);
                    }
                    _ => {}
                },
                StoreAction::ResetSearch => match self.search_bar_state {
                    SearchBarState::ExpandedWithFilters | SearchBarState::CollapsedWithFilters => {
                        self.search_bar_state = SearchBarState::ExpandedWithoutFilters;
                        self.search_bar(id!(search_bar)).expand(cx);
                        self.sorting(id!(sorting)).set_visible(cx, false);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
