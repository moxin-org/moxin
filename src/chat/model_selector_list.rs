use crate::{data::store::Store, shared::utils::format_model_size};
use makepad_widgets::*;
use moxin_mae::{MaeAgent, MaeBackend};
use moxin_protocol::data::DownloadedFile;
use std::collections::HashMap;

use super::model_selector_item::ModelSelectorItemWidgetRefExt;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;

    import crate::shared::styles::*;
    import crate::shared::widgets::*;
    import crate::chat::model_info::ModelInfo;
    import crate::chat::model_selector_item::ModelSelectorItem;

    AgentInfo = <View> {
        width: Fill,
        height: Fit,
        padding: 16,

        cursor: Hand,
        label = <Label> {
            draw_text:{
                text_style: <REGULAR_FONT>{font_size: 11},
                color: #000
            }
        }
    }

    ModelSelectorList = {{ModelSelectorList}} {
        flow: Down,
        model_template: <ModelSelectorItem> { content = <ModelInfo> {} }
        agent_template: <ModelSelectorItem> { content = <AgentInfo> {} }
        separator_template: <Line> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ModelSelectorList {
    #[redraw]
    #[rust]
    area: Area,

    #[walk]
    walk: Walk,

    #[layout]
    layout: Layout,

    #[live]
    model_template: Option<LivePtr>,
    #[live]
    agent_template: Option<LivePtr>,
    #[live]
    separator_template: Option<LivePtr>,

    #[live(true)]
    visible: bool,

    #[rust]
    items: ComponentMap<LiveId, WidgetRef>,

    #[rust]
    map_to_downloaded_files: HashMap<LiveId, DownloadedFile>,

    #[rust]
    total_height: Option<f64>,
}

impl Widget for ModelSelectorList {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        for (_, item) in self.items.iter_mut() {
            item.handle_event(cx, event, scope)
        }
        //     let actions = cx.capture_actions(|cx| item.handle_event(cx, event, scope));
        //     if let Some(fd) = item.as_view().finger_down(&actions) {
        //         if fd.tap_count == 1 {
        //             cx.widget_action(
        //                 widget_uid,
        //                 &scope.path,
        //                 ModelSelectorAction::Selected(
        //                     self.map_to_downloaded_files.get(id).unwrap().clone(),
        //                 ),
        //             );
        //         }
        //     }
        // }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let store = scope.data.get::<Store>().unwrap();
        cx.begin_turtle(walk, self.layout);

        if self.visible {
            self.draw_items(cx, store);
        }

        cx.end_turtle_with_area(&mut self.area);

        DrawStep::done()
    }
}

impl ModelSelectorList {
    fn draw_items(&mut self, cx: &mut Cx2d, store: &Store) {
        let mut models = store.downloads.downloaded_files.clone();
        models.sort_by(|a, b| b.downloaded_at.cmp(&a.downloaded_at));

        self.map_to_downloaded_files = HashMap::new();
        let mut total_height = 0.0;
        let models_count = models.len();
        for i in 0..models.len() {
            let item_id = LiveId(i as u64).into();
            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, self.model_template));
            self.map_to_downloaded_files
                .insert(item_id, models[i].clone());

            let caption = &models[i].file.name;

            let architecture = &models[i].model.architecture;
            let architecture_visible = !architecture.trim().is_empty();

            let param_size = &models[i].model.size;
            let param_size_visible = !param_size.trim().is_empty();

            let size = format_model_size(&models[i].file.size).unwrap_or("".to_string());
            let size_visible = !size.trim().is_empty();

            let mut icon_tick_visible = false;
            if let Some(loaded_model) = store.get_loaded_downloaded_file() {  
                icon_tick_visible = self.map_to_downloaded_files.get(&item_id).unwrap().file.id == loaded_model.file.id;
            }

            item_widget.apply_over(
                cx,
                live! {
                    content = {
                        label = { text: (caption) }
                        architecture_tag = { visible: (architecture_visible), caption = { text: (architecture) } }
                        params_size_tag = { visible: (param_size_visible), caption = { text: (param_size) } }
                        file_size_tag = { visible: (size_visible), caption = { text: (size) } }
                        icon_tick_tag = { visible: (icon_tick_visible) }
                    }
                },
            );

            item_widget.as_model_selector_item().set_model(models[i].clone());

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
            total_height += item_widget.view(id!(content)).area().rect(cx).size.y;
        }

        if models_count > 0 {
            let separator_id = LiveId(models_count as u64).into();
            let separator_widget = self
                .items
                .get_or_insert(cx, separator_id, |cx| WidgetRef::new_from_ptr(cx, self.separator_template));
            let _ = separator_widget.draw_all(cx, &mut Scope::empty());
            total_height += separator_widget.as_view().area().rect(cx).size.y;
        }

        let agents = MaeBackend::available_agents();
        for i in 0..agents.len() {
            let item_id = LiveId((models_count + 1 + i) as u64).into();
            let item_widget = self
                .items
                .get_or_insert(cx, item_id, |cx| WidgetRef::new_from_ptr(cx, self.agent_template));

            let agent_name = &agents[i].name();
            item_widget.apply_over(
                cx,
                live! {
                    content = { label = { text: (agent_name) } }
                },
            );
            item_widget.as_model_selector_item().set_agent(agents[i].clone());

            let _ = item_widget.draw_all(cx, &mut Scope::empty());
            total_height += item_widget.view(id!(content)).area().rect(cx).size.y;
        }


        self.total_height = Some(total_height);
    }
}

impl ModelSelectorListRef {
    pub fn get_height(&self) -> f64 {
        let Some(inner) = self.borrow_mut() else {
            return 0.0;
        };
        inner.total_height.unwrap_or(0.0)
    }
}
