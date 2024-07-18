use crate::{
    data::store::Store,
    shared::{actions::ChatAction, utils::format_model_size},
};
use makepad_widgets::*;

use super::model_selector_list::{ModelSelectorAction, ModelSelectorListWidgetExt};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    import crate::chat::model_info::ModelInfo;
    import crate::chat::model_selector_list::ModelSelectorList;

    ModelSelectorButton = <RoundedView> {
        width: Fill,
        height: 54,

        align: {x: 0.0, y: 0.5},
        padding: 16,

        draw_bg: {
            instance radius: 3.0,
            color: #F9FAFB,
        }

        cursor: Hand,

        choose = <View> {
            width: Fill,
            height: Fit,

            align: {x: 0.5, y: 0.5},

            label = <Label> {
                draw_text:{
                    text_style: <BOLD_FONT>{font_size: 11},
                    color: #000
                }
                text: "Choose a model"
            }
        }
        selected = <ModelInfo> {
            width: Fit,
            height: Fit,
            show_bg: false,
            visible: false,

            label = {
                draw_text: {
                    text_style: <BOLD_FONT>{font_size: 11},
                }
            }
        }
    }

    ModelSelectorOptions = <RoundedView> {
        width: Fill,
        height: 0,

        margin: { top: 5 },
        padding: 5,

        draw_bg: {
            instance radius: 3.0,
            color: #fff,
            border_color: #B6B6B6,
            border_width: 1.0,
        }

        list_container = <View> {
            width: Fill,
            height: 0,
            scroll_bars: <ScrollBars> {}

            list = <ModelSelectorList> {
                width: Fill,
                height: Fit,
            }
        }
    }

    ModelSelector = {{ModelSelector}} {
        width: Fill,
        height: Fit,

        flow: Down,

        button = <ModelSelectorButton> {}
        options = <ModelSelectorOptions> {}

        open_animation_progress: 0.0,
        animator: {
            open = {
                default: hide,
                show = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {open_animation_progress: 1.0}
                }
                hide = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {open_animation_progress: 0.0}
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelector {
    #[deref]
    view: View,

    #[rust]
    open: bool,

    #[animator]
    animator: Animator,

    #[live]
    open_animation_progress: f64,

    #[rust]
    hide_animation_timer: Timer,

    #[rust]
    options_list_height: Option<f64>,
}

impl Widget for ModelSelector {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);

        let store = scope.data.get::<Store>().unwrap();

        if let Hit::FingerDown(fd) = event.hits_with_capture_overload(cx, self.view(id!(button)).area(), true) {
            if no_options_to_display(store) {
                return;
            };
            if fd.tap_count == 1 {
                self.open = !self.open;

                if self.open {
                    let list = self.model_selector_list(id!(options.list_container.list));
                    let height = list.get_height();
                    if height > MAX_OPTIONS_HEIGHT {
                        self.options_list_height = Some(MAX_OPTIONS_HEIGHT);
                    } else {
                        self.options_list_height = Some(height);
                    }

                    self.view(id!(options)).apply_over(
                        cx,
                        live! {
                            height: Fit,
                        },
                    );

                    self.animator_play(cx, id!(open.show));
                } else {
                    self.hide_animation_timer = cx.start_timeout(0.3);
                    self.animator_play(cx, id!(open.hide));
                }
            }
        }

        if self.hide_animation_timer.is_event(event).is_some() {
            // When closing animation is done, hide the wrapper element
            self.view(id!(options)).apply_over(cx, live! { height: 0 });
            self.redraw(cx);
        }

        if self.animator_handle_event(cx, event).must_redraw() {
            if let Some(total_height) = self.options_list_height {
                let height = self.open_animation_progress * total_height;
                self.view(id!(options.list_container))
                    .apply_over(cx, live! {height: (height)});
                self.redraw(cx);
            }
        }

        if let Event::MouseDown(e) = event {
            if self.open {
                let hovered = self.view.area().rect(cx).contains(e.abs);
                if !hovered {
                    self.open = false;
                    self.hide_animation_timer = cx.start_timeout(0.3);
                    self.animator_play(cx, id!(open.hide));
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        let choose_label = self.label(id!(choose.label));

        if no_options_to_display(store) {
            choose_label.set_text("No Available Models");
            let color = vec3(0.596, 0.635, 0.702);
            choose_label.apply_over(
                cx,
                live! {
                    draw_text: {
                        color: (color)
                    }
                },
            );
        } else if no_active_model(store) {
            choose_label.set_text("Choose a Model");
            let color = vec3(0.0, 0.0, 0.0);
            choose_label.apply_over(
                cx,
                live! {
                    draw_text: {
                        color: (color)
                    }
                },
            );
        } else {
            self.update_selected_model_info(cx, store);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

const MAX_OPTIONS_HEIGHT: f64 = 400.0;

impl WidgetMatchEvent for ModelSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        // let store = scope.data.get::<Store>().unwrap();

        // if let Some(fd) = self.view(id!(button)).finger_down(&actions) {
        //     if no_options_to_display(store) {
        //         return;
        //     };
        //     if fd.tap_count == 1 {
        //         self.open = !self.open;

        //         if self.open {
        //             let list = self.model_selector_list(id!(options.list_container.list));
        //             let height = list.get_height();
        //             if height > MAX_OPTIONS_HEIGHT {
        //                 self.options_list_height = Some(MAX_OPTIONS_HEIGHT);
        //             } else {
        //                 self.options_list_height = Some(height);
        //             }

        //             self.view(id!(options)).apply_over(
        //                 cx,
        //                 live! {
        //                     height: Fit,
        //                 },
        //             );

        //             self.animator_play(cx, id!(open.show));
        //         } else {
        //             self.hide_animation_timer = cx.start_timeout(0.3);
        //             self.animator_play(cx, id!(open.hide));
        //         }
        //     }
        // }

        for action in actions {
            match action.as_widget_action().cast() {
                ModelSelectorAction::Selected(_) => {
                    self.hide_options(cx);
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                ChatAction::Start(_) => {
                    self.hide_options(cx);
                }
                _ => {}
            }
        }
    }
}

impl ModelSelector {
    fn hide_options(&mut self, cx: &mut Cx) {
        self.open = false;
        self.view(id!(options)).apply_over(cx, live! { height: 0 });
        self.animator_cut(cx, id!(open.hide));
    }

    fn update_selected_model_info(&mut self, cx: &mut Cx, store: &Store) {
        let Some(downloaded_file) = store.get_loaded_downloaded_file() else {
            return;
        };

        self.view(id!(choose)).apply_over(
            cx,
            live! {
                visible: false
            },
        );
        let filename = downloaded_file.file.name;

        let architecture = downloaded_file.model.architecture;
        let architecture_visible = !architecture.trim().is_empty();

        let param_size = downloaded_file.model.size;
        let param_size_visible = !param_size.trim().is_empty();

        let size = format_model_size(&downloaded_file.file.size).unwrap_or("".to_string());
        let size_visible = !size.trim().is_empty();

        self.view(id!(selected)).apply_over(
            cx,
            live! {
                visible: true
                label = { text: (filename) }
                architecture_tag = { visible: (architecture_visible), caption = { text: (architecture) }}
                params_size_tag = { visible: (param_size_visible), caption = { text: (param_size) }}
                file_size_tag = { visible: (size_visible), caption = { text: (size) }}
            },
        );
        self.redraw(cx);
    }

    fn deselect(&mut self, cx: &mut Cx) {
        self.open = false;
        self.view(id!(selected)).apply_over(
            cx,
            live! {
                visible: false
            },
        );

        self.view(id!(choose)).apply_over(
            cx,
            live! {
                visible: true
            },
        );
        self.redraw(cx);
    }
}

impl ModelSelectorRef {
    pub fn deselect(&mut self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.deselect(cx);
        }
    }
}

fn no_options_to_display(store: &Store) -> bool {
    store.downloads.downloaded_files.is_empty()
}

fn no_active_model(store: &Store) -> bool {
    store.get_loaded_downloaded_file().is_none()
}
