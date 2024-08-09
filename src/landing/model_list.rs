use crate::data::store::{Store, StoreAction};
use crate::landing::search_loading::SearchLoadingWidgetExt;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::landing::model_card::ModelCard;
    import crate::landing::search_loading::SearchLoading;

    ModelList = {{ModelList}} {
        width: Fill,
        height: Fill,

        flow: Overlay,

        content = <View> {
            width: Fill,
            height: Fill,
            list = <PortalList> {
                width: Fill,
                height: Fill,

                // We need this setting because we will have modal dialogs that should
                // "capture" the events, so we don't want to handle them here.
                capture_overload: false,

                Model = <ModelCard> {
                    margin: {bottom: 30},
                }
            }
        }

        loading = <View> {
            width: Fill,
            height: Fill,
            visible: false,

            show_bg: true,
            draw_bg: {
                color: #FFFE,
            }
            search_loading = <SearchLoading> {}
        }

        search_error = <View> {
            width: Fill,
            height: Fill,
            visible: false,
            align: {x: 0.5, y: 0.5},

            <Label> {
                draw_text:{
                    text_style: <REGULAR_FONT>{font_size: 13},
                    color: #000
                }
                text: "Error fetching models. Please check your internet connection and try again."
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelList {
    #[deref]
    view: View,

    #[rust]
    loading_delay: Timer,
}

impl Widget for ModelList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        if let Event::Signal = event {
            self.loading_delay = cx.start_timeout(0.2);
        }

        if self.loading_delay.is_event(event).is_some() {
            self.update_loading_and_error_message(cx, scope);
        }
        self.redraw(cx);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let models = &store.search.models;
        let models_count = models.len();

        while let Some(view_item) = self.view.draw_walk(cx, &mut Scope::empty(), walk).step() {
            if let Some(mut list) = view_item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, models_count);
                while let Some(item_id) = list.next_visible_item(cx) {
                    let item = list.item(cx, item_id, live_id!(Model)).unwrap();

                    if item_id < models_count {
                        let model = &models[item_id];
                        let mut model_with_download_info =
                            store.add_download_info_to_model(model);
                        item.draw_all(cx, &mut Scope::with_data(&mut model_with_download_info));
                    }
                }
            }
        }

        DrawStep::done()
    }
}

#[derive(Clone, DefaultNone, Debug)]
pub enum ModelListAction {
    None,
    ScrolledAtTop,
    ScrolledNotAtTop,
}

const SCROLLING_AT_TOP_THRESHOLD: f64 = -30.0;

impl WidgetMatchEvent for ModelList {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let portal_list = self.portal_list(id!(list));

        for action in actions.iter() {
            match action.as_widget_action().cast() {
                StoreAction::Search(_) | StoreAction::ResetSearch => {
                    self.view(id!(search_error)).set_visible(false);
                    self.view(id!(loading)).set_visible(true);
                    self.search_loading(id!(search_loading)).animate(cx);
                    portal_list.set_first_id_and_scroll(0, 0.0);

                    self.redraw(cx);
                }
                _ => {}
            }
        }

        if portal_list.scrolled(actions) {
            let widget_uid = self.widget_uid();
            if portal_list.first_id() == 0
                && portal_list.scroll_position() > SCROLLING_AT_TOP_THRESHOLD
            {
                cx.widget_action(widget_uid, &scope.path, ModelListAction::ScrolledAtTop);
            } else {
                cx.widget_action(widget_uid, &scope.path, ModelListAction::ScrolledNotAtTop);
            }
        }
    }
}

impl ModelList {
    fn update_loading_and_error_message(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let store = scope.data.get::<Store>().unwrap();
        let is_loading = store.search.is_pending();
        self.view(id!(loading)).set_visible(is_loading);
        if is_loading {
            self.search_loading(id!(search_loading)).animate(cx);
        } else {
            self.search_loading(id!(search_loading)).stop_animation();
        }

        let is_errored = store.search.was_error();
        self.view(id!(search_error)).set_visible(is_errored);
    }
}
