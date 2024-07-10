use crate::chat::chat_panel::ChatPanelAction;
use crate::chat::delete_chat_modal::{DeleteChatAction, DeleteChatModalWidgetRefExt};
use crate::data::downloads::DownloadPendingNotification;
use crate::data::store::*;
use crate::landing::model_card::{ModelCardViewAllModalWidgetRefExt, ViewAllModalAction};
use crate::landing::model_files_item::ModelFileItemAction;
use crate::my_models::delete_model_modal::{DeleteModelAction, DeleteModelModalWidgetRefExt};
use crate::my_models::model_info_modal::{ModelInfoAction, ModelInfoModalWidgetRefExt};
use crate::shared::actions::{ChatAction, DownloadAction, TooltipAction};
use crate::shared::download_notification_popup::{
    DownloadNotificationPopupWidgetRefExt, DownloadResult, PopupAction,
};
use crate::shared::portal::{PortalViewWidgetRefExt, PortalWidgetRefExt};
use crate::shared::tooltip::TooltipWidgetRefExt;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::portal::*;
    import crate::shared::modal::*;
    import crate::shared::widgets::SidebarMenuButton;
    import crate::shared::download_notification_popup::DownloadNotificationPopup;
    import crate::shared::tooltip::Tooltip;
    import crate::landing::landing_screen::LandingScreen;
    import crate::landing::model_card::ModelCardViewAllModal;
    import crate::chat::chat_screen::ChatScreen;
    import crate::my_models::my_models_screen::MyModelsScreen;
    import crate::my_models::delete_model_modal::DeleteModelModal;
    import crate::chat::delete_chat_modal::DeleteChatModal;
    import crate::my_models::model_info_modal::ModelInfoModal;


    ICON_DISCOVER = dep("crate://self/resources/icons/discover.svg")
    ICON_CHAT = dep("crate://self/resources/icons/chat.svg")
    ICON_MY_MODELS = dep("crate://self/resources/icons/my_models.svg")

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1440, 1024)},
            pass: {clear_color: #fff}

            body = {
                flow: Overlay
                width: Fill,
                height: Fill,

                root = <View> {
                    width: Fill,
                    height: Fill,

                    sidebar_menu = <RoundedView> {
                        width: 100,
                        flow: Down, spacing: 20.0,
                        padding: { top: 80 }

                        align: {x: 0.5, y: 0.0},

                        show_bg: true,
                        draw_bg: {
                            color: (SIDEBAR_BG_COLOR),
                            instance radius: 0.0,
                            border_color: #EAECF0,
                            border_width: 1.2,
                        }

                        discover_tab = <SidebarMenuButton> {
                            animator: {selected = {default: on}}
                            text: "Discover",
                            draw_icon: {
                                svg_file: (ICON_DISCOVER),
                            }
                        }
                        chat_tab = <SidebarMenuButton> {
                            text: "Chat",
                            draw_icon: {
                                svg_file: (ICON_CHAT),
                            }
                        }
                        my_models_tab = <SidebarMenuButton> {
                            text: "My Models",
                            draw_icon: {
                                svg_file: (ICON_MY_MODELS),
                            }
                        }
                    }

                    application_pages = <View> {
                        margin: 0.0,
                        padding: 0.0,

                        flow: Overlay,

                        width: Fill,
                        height: Fill,

                        discover_frame = <LandingScreen> {visible: true}
                        chat_frame = <ChatScreen> {visible: false}
                        my_models_frame = <MyModelsScreen> {visible: false}
                    }
                }

                portal_root = <Portal> {
                    modal_model_card_view_all_portal_view = <PortalView> {
                        modal = <Modal> {
                            content = {
                                model_card_view_all_modal = <ModelCardViewAllModal> {}
                            }
                        }
                    }

                    modal_delete_model_portal_view = <PortalView> {
                        modal = <Modal> {
                            content = {
                                delete_model_modal = <DeleteModelModal> {}
                            }
                        }
                    }

                    modal_delete_chat_portal_view = <PortalView> {
                        modal = <Modal> {
                            content = {
                                delete_chat_modal = <DeleteChatModal> {}
                            }
                        }
                    }

                    modal_model_info_portal_view = <PortalView> {
                        modal = <Modal> {
                            content = {
                                model_info_modal = <ModelInfoModal> {}
                            }
                        }
                    }

                    popup_download_success_portal_view = <PortalView> {
                        align: {x: 1, y: 0}
                        popup_download_success = <DownloadNotificationPopup> {}
                    }

                    tooltip_portal_view = <PortalView> {
                        tooltip = <Tooltip> {}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    store: Store,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);

        crate::shared::live_design(cx);
        crate::landing::live_design(cx);
        crate::chat::live_design(cx);
        crate::my_models::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // Process all possible store incoming events
        if let Event::Signal = event {
            self.store.process_event_signal();
            self.notify_downloaded_files(cx);
            self.ui.redraw(cx);
        }

        let scope = &mut Scope::with_data(&mut self.store);
        self.ui.handle_event(cx, event, scope);
        self.match_event(cx, event);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        self.ui
            .radio_button_set(ids!(
                sidebar_menu.discover_tab,
                sidebar_menu.chat_tab,
                sidebar_menu.my_models_tab,
            ))
            .selected_to_visible(
                cx,
                &self.ui,
                actions,
                ids!(
                    application_pages.discover_frame,
                    application_pages.chat_frame,
                    application_pages.my_models_frame,
                ),
            );

        for action in actions.iter() {
            match action.as_widget_action().cast() {
                StoreAction::Search(keywords) => {
                    self.store.search.load_search_results(keywords);
                }
                StoreAction::ResetSearch => {
                    self.store.search.load_featured_models();
                }
                StoreAction::Sort(criteria) => {
                    self.store.search.sort_models(criteria);
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                ModelFileItemAction::Download(file_id) => {
                    let (model, file) = self.store.get_model_and_file_download(&file_id);
                    self.store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                DownloadAction::Play(file_id) => {
                    let (model, file) = self.store.get_model_and_file_download(&file_id);
                    self.store.downloads.download_file(model, file);
                    self.ui.redraw(cx);
                }
                DownloadAction::Pause(file_id) => {
                    self.store.downloads.pause_download_file(file_id);
                    self.ui.redraw(cx);
                }
                DownloadAction::Cancel(file_id) => {
                    self.store.downloads.cancel_download_file(file_id);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            // Set modal viewall model id
            if let ViewAllModalAction::ModelSelected(model_id) = action.as_widget_action().cast() {
                let mut modal = self
                    .ui
                    .model_card_view_all_modal(id!(model_card_view_all_modal));
                modal.set_model_id(model_id);
                // TODO: Hack for error that when you first open the modal, doesnt draw until an event
                // this forces the entire ui to rerender, still weird that only happens the first time.
                self.ui.redraw(cx);
            }

            // Set modal viewall model id
            if let DeleteModelAction::FileSelected(file_id) = action.as_widget_action().cast() {
                let mut modal = self.ui.delete_model_modal(id!(delete_model_modal));
                modal.set_file_id(file_id);
                // TODO: Hack for error that when you first open the modal, doesnt draw until an event
                // this forces the entire ui to rerender, still weird that only happens the first time.
                self.ui.redraw(cx);
            }

            // Set modal viewall model id
            if let DeleteChatAction::ChatSelected(chat_id) = action.as_widget_action().cast() {
                let mut modal = self.ui.delete_chat_modal(id!(delete_chat_modal));
                modal.set_chat_id(chat_id);
                // TODO: Hack for error that when you first open the modal, doesnt draw until an event
                // this forces the entire ui to rerender, still weird that only happens the first time.
                self.ui.redraw(cx);
            }

            if let ModelInfoAction::FileSelected(file_id) = action.as_widget_action().cast() {
                let mut modal = self.ui.model_info_modal(id!(model_info_modal));
                modal.set_file_id(file_id);
                // TODO: Hack for error that when you first open the modal, doesnt draw until an event
                // this forces the entire ui to rerender, still weird that only happens the first time.
                self.ui.redraw(cx);
            }

            if let ChatAction::Start(_) = action.as_widget_action().cast() {
                let chat_radio_button = self.ui.radio_button(id!(chat_tab));
                chat_radio_button.select(cx, &mut Scope::empty());
            }

            if let PopupAction::NavigateToMyModels = action.as_widget_action().cast() {
                let my_models_radio_button = self.ui.radio_button(id!(my_models_tab));
                my_models_radio_button.select(cx, &mut Scope::empty());
            }

            if let ChatPanelAction::NavigateToDiscover = action.as_widget_action().cast() {
                let discover_radio_button = self.ui.radio_button(id!(discover_tab));
                discover_radio_button.select(cx, &mut Scope::empty());
            }

            if let ChatAction::Resume = action.as_widget_action().cast() {
                let chat_radio_button = self.ui.radio_button(id!(chat_tab));
                chat_radio_button.select(cx, &mut Scope::empty());
            }

            match action.as_widget_action().cast() {
                TooltipAction::Show(text, pos) => {
                    let mut tooltip = self.ui.tooltip(id!(tooltip));
                    tooltip.set_text(&text);

                    let tooltip_portal_view = self.ui.portal_view(id!(tooltip_portal_view));
                    tooltip_portal_view.apply_over_and_redraw(cx, live!{
                        padding: { left: (pos.x), top: (pos.y) }
                    });
                    
                    let mut portal = self.ui.portal(id!(portal_root));
                    let _ = portal.show_portal_view_by_id(cx, live_id!(tooltip_portal_view));
                }
                TooltipAction::Hide => {
                    let mut portal = self.ui.portal(id!(portal_root));
                    let _ = portal.close(cx);
                }
                _ => {}
            }
        }
    }
}

impl App {
    fn notify_downloaded_files(&mut self, cx: &mut Cx) {
        if let Some(notification) = self.store.downloads.next_download_notification() {
            let mut popup = self
                .ui
                .download_notification_popup(id!(popup_download_success));

            match notification {
                DownloadPendingNotification::DownloadedFile(file) => {
                    popup.set_data(&file, DownloadResult::Success);
                }
                DownloadPendingNotification::DownloadErrored(file) => {
                    popup.set_data(&file, DownloadResult::Failure);
                }
            }

            let mut portal = self.ui.portal(id!(portal_root));
            let _ = portal.show_portal_view_by_id(cx, live_id!(popup_download_success_portal_view));
        }
    }
}
