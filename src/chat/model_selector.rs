use crate::{
    data::{chats::chat::ChatEntity, store::Store},
    shared::{
        actions::ChatAction,
        utils::{format_model_size, hex_rgb_color},
    },
};
use makepad_widgets::*;

use super::{
    model_selector_item::ModelSelectorAction,
    model_selector_list::ModelSelectorListWidgetExt,
    model_selector_loading::ModelSelectorLoadingWidgetExt,
};

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;

    import crate::chat::model_info::ModelInfo;
    import crate::chat::model_selector_list::ModelSelectorList;
    import crate::chat::model_selector_loading::ModelSelectorLoading;

    ICON_DROP = dep("crate://self/resources/images/drop_icon.png")


    ModelSelectorButton = <RoundedView> {
        width: Fill,
        height: 54,
        flow: Overlay,

        loading = <ModelSelectorLoading> {
            width: Fill,
            height: Fill,
            visible: false,
        }

        draw_bg: {
            instance radius: 3.0,
            color: #F9FAFB,
        }

        <View> {
            width: Fill,
            height: Fill,
            flow: Right,

            align: {x: 0.0, y: 0.5},
            padding: {left: 16, right: 16, top: 0, bottom: 0},

            cursor: Hand,

            content = <View> {
                width: Fill,
                height: Fit,
                flow: Overlay,
                padding: {left: 16, top: 0, bottom: 0, right: 0},

                choose = <View> {
                    width: Fill,
                    height: Fit,

                    align: {x: 0.0, y: 0.5},
                    padding: 16,

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

                    padding: 0,

                    label = {
                        draw_text: {
                            text_style: <BOLD_FONT>{font_size: 11},
                        }
                    }
                }
            }

            icon_drop = <RoundedView> {
                width: Fit,
                height: Fit,
                align: {x: 1.0, y: 0.5},
                margin: {left: 10, right: 6},
                visible: false,

                icon = <RotatedImage> {
                    height: 14,
                    width: 14,
                    source: (ICON_DROP),
                    draw_bg: {
                        rotation: 0.0
                    }
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
            border_color: #D0D5DD,
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
        rotate_animation_progress: 0.0
        animator: {
            open = {
                default: hide,
                show = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {open_animation_progress: 1.0, rotate_animation_progress: 1.0}
                }
                hide = {
                    redraw: true,
                    from: {all: Forward {duration: 0.3}}
                    ease: ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {open_animation_progress: 0.0, rotate_animation_progress: 0.0}
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

    #[live]
    rotate_animation_progress: f64,

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

        if let Hit::FingerDown(fd) =
            event.hits_with_capture_overload(cx, self.view(id!(button)).area(), true)
        {
            if !options_to_display(store) {
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

                let rotate_angle = self.rotate_animation_progress * std::f64::consts::PI;
                self.view(id!(icon_drop.icon))
                    .apply_over(cx, live! {draw_bg: {rotation: (rotate_angle)}});

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

        self.update_loading_model_state(cx, store);

        if !options_to_display(store) {
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
            self.view(id!(icon_drop)).apply_over(
                cx,
                live!{
                    visible: false
                });
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

            self.view(id!(icon_drop)).apply_over(
                cx,
                live!{
                    visible: true
                });
        } else {
            self.update_selected_model_info(cx, store);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

const MAX_OPTIONS_HEIGHT: f64 = 400.0;

impl WidgetMatchEvent for ModelSelector {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let store = scope.data.get::<Store>().unwrap();

        if let Some(fd) = self.view(id!(button)).finger_down(&actions) {
            if options_to_display(store) && fd.tap_count == 1 {
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

        for action in actions {
            match action.cast() {
                ModelSelectorAction::ModelSelected(_) | ModelSelectorAction::AgentSelected(_) => {
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
        self.view(id!(icon_drop.icon))
            .apply_over(cx, live! {draw_bg: {rotation: (0.0)}});
        self.animator_cut(cx, id!(open.hide));
        self.redraw(cx);
    }

    fn update_loading_model_state(&mut self, cx: &mut Cx, store: &Store) {
        if store.chats.model_loader.is_loading() {
            self.model_selector_loading(id!(loading))
                .show_and_animate(cx);
        } else {
            self.model_selector_loading(id!(loading)).hide();
        }
    }

    fn update_selected_model_info(&mut self, cx: &mut Cx, store: &Store) {
        self.view(id!(choose)).set_visible(false);

        let is_loading = store.chats.model_loader.is_loading();
        let loaded_file = store.chats.loaded_model.as_ref();

        let chat_entity = store
            .chats
            .get_current_chat()
            .and_then(|c| c.borrow().last_used_entity.clone());

        let file = match chat_entity {
            Some(ChatEntity::ModelFile(file_id)) => store.downloads.get_file(&file_id).cloned(),
            _ => loaded_file.cloned(),
        };

        if let Some(file) = file {
            let selected_view = self.view(id!(selected));
            selected_view.set_visible(true);

            let text_color = if Some(&file.id) == loaded_file.map(|f| &f.id) {
                hex_rgb_color(0x000000)
            } else {
                hex_rgb_color(0x667085)
            };

            let caption = if is_loading {
                format!("Loading {}", file.name.trim())
            } else {
                file.name.trim().to_string()
            };

            let file_size = format_model_size(file.size.trim()).unwrap_or("".into());
            let is_file_size_visible = !file_size.is_empty() && !is_loading;

            selected_view.apply_over(
                cx,
                live! {
                    label = { text: (caption), draw_text: { color: (text_color) }}
                    file_size_tag = { visible: (is_file_size_visible), caption = { text: (file_size), draw_text: { color: (text_color) }}}
                },
            );

            if let Some(model) = store.downloads.get_model_by_file_id(&file.id) {
                let architecture = model.architecture.trim();
                let params_size = model.size.trim();
                let is_architecture_visible = !architecture.is_empty() && !is_loading;
                let is_params_size_visible = !params_size.is_empty() && !is_loading;

                selected_view.apply_over(
                    cx,
                    live! {
                        architecture_tag = { visible: (is_architecture_visible), caption = { text: (architecture), draw_text: { color: (text_color) }}}
                        params_size_tag = { visible: (is_params_size_visible), caption = { text: (params_size), draw_text: { color: (text_color) }}}
                    },
                );
            }
        }

        self.view(id!(icon_drop)).apply_over(
            cx,
            live!{
                visible: true
            });

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

fn options_to_display(store: &Store) -> bool {
    !store.downloads.downloaded_files.is_empty()
}

fn no_active_model(store: &Store) -> bool {
    store.get_loaded_downloaded_file().is_none() && store.get_loading_file().is_none()
}
