pub mod agent_button;
pub mod chat_history;
pub mod chat_history_card;
pub mod chat_history_card_options;
pub mod chat_line;
pub mod chat_line_loading;
pub mod chat_panel;
pub mod chat_params;
pub mod chat_screen;
pub mod delete_chat_modal;
pub mod model_info;
pub mod model_selector;
pub mod model_selector_item;
pub mod model_selector_list;
pub mod model_selector_loading;
pub mod prompt_input;
pub mod shared;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    agent_button::live_design(cx);
    prompt_input::live_design(cx);
    chat_history_card::live_design(cx);
    chat_history::live_design(cx);
    chat_line_loading::live_design(cx);
    chat_line::live_design(cx);
    chat_panel::live_design(cx);
    chat_params::live_design(cx);
    chat_screen::live_design(cx);
    model_info::live_design(cx);
    model_selector_list::live_design(cx);
    model_selector_item::live_design(cx);
    model_selector::live_design(cx);
    model_selector_loading::live_design(cx);
    shared::live_design(cx);
    delete_chat_modal::live_design(cx);
    chat_history_card_options::live_design(cx);
}
