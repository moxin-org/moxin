use crate::chat::chat_panel::ChatPanelAction;
use crate::data::downloads::DownloadPendingNotification;
use crate::data::store::*;
use crate::landing::model_files_item::ModelFileItemAction;
use crate::shared::actions::{ChatAction, DownloadAction};
use crate::shared::download_notification_popup::{
    DownloadNotificationPopupAction, DownloadNotificationPopupWidgetRefExt, DownloadResult, PopupAction
};
use crate::shared::popup_notification::PopupNotificationWidgetRefExt;
use makepad_widgets::*;

live_design! {
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_draw::shader::std::*;

    import crate::shared::styles::*;
    import crate::shared::popup_notification::*;
    import crate::shared::widgets::SidebarMenuButton;
    import crate::shared::download_notification_popup::DownloadNotificationPopup;
    import crate::shared::desktop_buttons::MoxinDesktopButton;

    import crate::landing::landing_screen::LandingScreen;
    import crate::landing::model_card::ModelCardViewAllModal;
    import crate::chat::chat_screen::ChatScreen;
    import crate::my_models::my_models_screen::MyModelsScreen;


    ICON_DISCOVER = dep("crate://self/resources/icons/discover.svg")
    ICON_CHAT = dep("crate://self/resources/icons/chat.svg")
    ICON_MY_MODELS = dep("crate://self/resources/icons/my_models.svg")

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1440.0, 1024.0)},
            pass: {clear_color: #fff}

            caption_bar = {
                caption_label = <View> {} // empty view to remove the default caption label
                windows_buttons = <View> {
                    visible: false,
                    width: Fit, height: Fit,
                    min = <MoxinDesktopButton> {draw_bg: {button_type: WindowsMin}}
                    max = <MoxinDesktopButton> {draw_bg: {button_type: WindowsMax}}
                    close = <MoxinDesktopButton> {draw_bg: {button_type: WindowsClose}}
                }
            }

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

                popup_notification = <PopupNotification> {
                    content: {
                        popup_download_notification = <DownloadNotificationPopup> {}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    store: Store,

    #[rust]
    screen_width: f64,

    #[rust]
    screen_length: f64,
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

impl LiveHook for App {
    fn before_apply(&mut self, cx: &mut Cx, _apply: &mut Apply, _index: usize, _nodes: &[LiveNode]) {
        self.ui.as_window().set_inner_size(cx, vec2(1440.0, 1024.0).into());
    }
    fn after_new_from_doc(&mut self, cx:&mut Cx){
        // self.ui.window(id!(window)).set_inner_size(cx, vec2(1440.0, 1024.0).into());
        self.ui.as_window().set_inner_size(cx, vec2(1440.0, 1024.0).into());
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

        // self.ui.window(id!(window)).set_inner_size(cx, vec2(1440.0, 1024.0).into());
        self.ui.as_window().set_inner_size(cx, vec2(1440.0, 1024.0).into());

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
                    self.store.downloads.pause_download_file(&file_id);
                    self.ui.redraw(cx);
                }
                DownloadAction::Cancel(file_id) => {
                    self.store.downloads.cancel_download_file(&file_id);
                    self.ui.redraw(cx);
                }
                _ => {}
            }

            match action.as_widget_action().cast() {
                WindowAction::WindowGeomChange(ce) => {
                    self.screen_width = ce.new_geom.inner_size.x;
                    self.screen_length = ce.new_geom.inner_size.y;
                    log!("the width of the window: {}, the length of the window: {}", self.screen_width, self.screen_length);
                    self.store.preferences.set_size_of_window((self.screen_width, self.screen_length));
                    self.ui.redraw(cx);
                },
                _ => {}
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

            if matches!(
                action.as_widget_action().cast(),
                DownloadNotificationPopupAction::ActionLinkClicked
                    | DownloadNotificationPopupAction::CloseButtonClicked
            ) {
                self.ui.popup_notification(id!(popup_notification)).close(cx);
            }
        }
    }
}

impl App {
    fn notify_downloaded_files(&mut self, cx: &mut Cx) {
        if let Some(notification) = self.store.downloads.next_download_notification() {
            let mut popup = self
                .ui
                .download_notification_popup(id!(popup_download_notification));

            match notification {
                DownloadPendingNotification::DownloadedFile(file) => {
                    popup.set_data(&file, DownloadResult::Success);
                }
                DownloadPendingNotification::DownloadErrored(file) => {
                    popup.set_data(&file, DownloadResult::Failure);
                }
            }

            self.ui.popup_notification(id!(popup_notification)).open(cx);
        }
    }
}
