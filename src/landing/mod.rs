pub mod agent_list;
pub mod download_item;
pub mod downloads;
pub mod landing_screen;
pub mod model_card;
pub mod model_files;
pub mod model_files_item;
pub mod model_files_list;
pub mod model_files_tags;
pub mod model_list;
pub mod search_bar;
pub mod search_loading;
pub mod shared;
pub mod sorting;

use makepad_widgets::Cx;

pub fn live_design(cx: &mut Cx) {
    shared::live_design(cx);
    agent_list::live_design(cx);
    model_files_tags::live_design(cx);
    model_files_item::live_design(cx);
    model_files_list::live_design(cx);
    model_files::live_design(cx);
    model_card::live_design(cx);
    model_list::live_design(cx);
    landing_screen::live_design(cx);
    search_bar::live_design(cx);
    search_loading::live_design(cx);
    sorting::live_design(cx);
    downloads::live_design(cx);
    download_item::live_design(cx);
}
