use std::path::PathBuf;

use moxin_protocol::data::FileID;
use serde::{Deserialize, Serialize};

use super::filesystem::{
    setup_preferences_folder, setup_model_downloads_folder, read_from_file, write_to_file,
};
const PREFERENCES_FILENAME: &str = "preferences.json";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Preferences {
    pub current_chat_model: Option<FileID>,
    #[serde(default)]
    pub downloaded_files_dir: PathBuf,

    pub open_ai_api_key: Option<String>,
    pub serper_api_key: Option<String>,
}

impl Preferences {
    pub fn load() -> Self {
            let preferences_path = preferences_path();

            if let Ok(json) = read_from_file(preferences_path) {
                if let Ok(mut preferences) = serde_json::from_str::<Preferences>(&json) {
                    // Check if the downloaded_files_dir exists, if not, create it
                    if !preferences.downloaded_files_dir.exists() {
                        preferences.downloaded_files_dir = setup_model_downloads_folder();
                    }
                    return preferences;
                }
            }

            // If no preferences file exists, create default preferences
            Self {
                current_chat_model: None,
                downloaded_files_dir: setup_model_downloads_folder(),
                open_ai_api_key: None,
                serper_api_key: None,
            }
        }

    pub fn save(&self) {
        let json = serde_json::to_string(&self).unwrap();
        match write_to_file(preferences_path(), &json) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to write to the file: {:?}", e),
        }
    }

    pub fn set_current_chat_model(&mut self, file: FileID) {
        self.current_chat_model = Some(file);
        self.save();
    }

    pub fn set_downloaded_files_dir(&mut self, path: PathBuf) {
        self.downloaded_files_dir = path;
        self.save();
    }
}

fn preferences_path() -> PathBuf {
    let preference_dir = setup_preferences_folder();
    preference_dir.join(PREFERENCES_FILENAME)
}
