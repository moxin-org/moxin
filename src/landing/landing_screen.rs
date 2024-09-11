use crate::data::store::Store;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::search_bar::SearchBar;
    import crate::landing::model_list::ModelList;
    import crate::landing::downloads::Downloads;

    Heading = <View> {
        width: Fill,
        height: Fit,

        heading_no_filters = <View> {
            width: Fit,
            height: 50,
            align: {y: 0.5},

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
            align: {y: 0.5},

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

#[derive(Live, LiveHook, Widget)]
pub struct LandingScreen {
    #[deref]
    view: View,
}

impl Widget for LandingScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let search = &scope.data.get::<Store>().unwrap().search;
        if search.is_pending() || search.was_error() {
            self.view(id!(heading_with_filters)).set_visible(false);
            self.view(id!(heading_no_filters)).set_visible(false);
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
