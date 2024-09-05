use super::chats::chat::{ChatEntity, ChatID};
use super::filesystem::project_dirs;
use super::preferences::Preferences;
use super::search::SortCriteria;
use super::{chats::Chats, downloads::Downloads, search::Search};
use anyhow::Result;
use chrono::{DateTime, Utc};
use makepad_widgets::{DefaultNone, SignalToUI};
use moxin_backend::Backend;
use moxin_mae::{MaeAgent, MaeBackend};
use moxin_protocol::data::{Author, DownloadedFile, File, FileID, Model, ModelID, PendingDownload};
use std::collections::HashMap;
use std::rc::Rc;

pub const DEFAULT_MAX_DOWNLOAD_THREADS: usize = 3;

#[derive(Clone, DefaultNone, Debug)]
pub enum StoreAction {
    Search(String),
    ResetSearch,
    Sort(SortCriteria),
    None,
}

#[derive(Clone, Debug)]
pub struct FileWithDownloadInfo {
    pub file: File,
    pub download: Option<PendingDownload>,
}

#[derive(Clone, Debug)]
pub struct ModelWithDownloadInfo {
    pub model_id: ModelID,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    pub author: Author,
    pub like_count: u32,
    pub download_count: u32,
    pub files: Vec<FileWithDownloadInfo>,
}
pub struct Store {
    /// This is the backend representation, including the sender and receiver ends of the channels to
    /// communicate with the backend thread.
    pub backend: Rc<Backend>,

    pub mae_backend: Rc<MaeBackend>,

    pub search: Search,
    pub downloads: Downloads,
    pub chats: Chats,
    pub preferences: Preferences,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        let mut preferences = Preferences::load();
        let app_data_dir = project_dirs().data_dir();

        let backend = Rc::new(Backend::new(
            app_data_dir,
            preferences.downloaded_files_dir.clone(),
            DEFAULT_MAX_DOWNLOAD_THREADS,
        ));

        // Temporary solution to load API keys from environment variables
        if std::env::var("MODEL_API_KEY").is_ok() {
            preferences.open_ai_api_key = Some(std::env::var("MODEL_API_KEY").unwrap());
        }
        if std::env::var("SERPER_API_KEY").is_ok() {
            preferences.serper_api_key = Some(std::env::var("SERPER_API_KEY").unwrap());
        }
        if std::env::var("AGENTOPS_API_KEY").is_ok() {
            preferences.agentops_api_key = Some(std::env::var("AGENTOPS_API_KEY").unwrap());
        }
        preferences.save();
        // End of temporary solution

        let mut options: HashMap<String, String> = HashMap::new();
        if let Some(api_key) = &preferences.open_ai_api_key {
            options.insert("model_api_key".to_string(), api_key.to_string());
        } else {
            eprintln!("No OpenAI API key found");
        }
        if let Some(api_key) = &preferences.serper_api_key {
            options.insert("serper_api_key".to_string(), api_key.to_string());
        } else {
            eprintln!("No Serper API key found");
        }
        if let Some(api_key) = &preferences.agentops_api_key {
            options.insert("agentops_api_key".to_string(), api_key.to_string());
        } else {
            eprintln!("No AgentOps API key found");
        }

        //let mae_backend = Rc::new(MaeBackend::new(options));
        let mae_backend = Rc::new(MaeBackend::new_fake());

        let mut store = Self {
            backend: backend.clone(),
            mae_backend: mae_backend.clone(),

            search: Search::new(backend.clone()),
            downloads: Downloads::new(backend.clone()),
            chats: Chats::new(backend, mae_backend),
            preferences,
        };

        store.downloads.load_downloaded_files();
        store.downloads.load_pending_downloads();

        store.chats.load_chats();
        store.init_current_chat();

        store.search.load_featured_models();
        store
    }

    pub fn load_model(&mut self, file: &File) {
        self.chats.load_model(file);
    }

    fn update_load_model(&mut self) {
        if self.chats.model_loader.is_loaded() {
            self.chats.loaded_model = self
                .chats
                .model_loader
                .file_id()
                .map(|id| self.downloads.get_file(&id))
                .flatten()
                .cloned();
        }

        if let Some(file) = &self.chats.loaded_model {
            self.preferences.set_current_chat_model(file.id.clone());

            // If there is no chat, create an empty one
            if self.chats.get_current_chat().is_none() {
                self.chats.create_empty_chat();
            }
        }
    }

    pub fn send_chat_message(&mut self, prompt: String, regenerate_from: Option<usize>) {
        if let Some(mut chat) = self.chats.get_current_chat().map(|c| c.borrow_mut()) {
            let wanted_file = self
                .chats
                .get_chat_file_id(&mut chat)
                .map(|file_id| self.downloads.get_file(&file_id))
                .flatten();

            if let Some(file) = wanted_file {
                if let Some(message_id) = regenerate_from {
                    chat.remove_messages_from(message_id);
                }
                chat.send_message_to_model(
                    prompt,
                    file,
                    self.chats.model_loader.clone(),
                    &self.backend
                );
                chat.save();
            }
        }
    }

    pub fn agents_list(&self) -> Vec<MaeAgent> {
        MaeBackend::available_agents()
    }

    pub fn send_agent_message(&self, agent: MaeAgent, prompt: String, regenerate_from: Option<usize>) {
        if let Some(mut chat) = self.chats.get_current_chat().map(|c| c.borrow_mut()) {
            if let Some(message_id) = regenerate_from {
                chat.remove_messages_from(message_id);
            }
            chat.send_message_to_agent(agent, prompt, &self.mae_backend);
        }
    }

    pub fn edit_chat_message(&mut self, message_id: usize, updated_message: String) {
        if let Some(mut chat) = self.chats.get_current_chat().map(|c| c.borrow_mut()) {
            chat.edit_message(message_id, updated_message);
        }
    }

    pub fn get_loading_file(&self) -> Option<&File> {
        self.chats
            .model_loader
            .get_loading_file_id()
            .map(|file_id| self.downloads.get_file(&file_id))
            .flatten()
    }

    pub fn get_loaded_downloaded_file(&self) -> Option<DownloadedFile> {
        if let Some(file) = &self.chats.loaded_model {
            self.downloads
                .downloaded_files
                .iter()
                .find(|d| d.file.id == file.id)
                .cloned()
        } else {
            None
        }
    }

    pub fn get_chat_entity_name(&self, chat_id: ChatID) -> Option<String> {
        let Some(chat) = self.chats.get_chat_by_id(chat_id) else {
            return None;
        };

        match chat.borrow().associated_entity {
            Some(ChatEntity::ModelFile(ref file_id)) => self
                .downloads
                .downloaded_files
                .iter()
                .find(|df| df.file.id == *file_id)
                .map(|df| Some(df.file.name.clone()))?,
            Some(ChatEntity::Agent(agent)) => Some(agent.name()),
            None => {
                // Fallback to loaded model if exists
                self.chats.loaded_model.as_ref().map(|m| m.name.clone())
            },
        }
    }

    /// This function combines the search results information for a given model
    /// with the download information for the files of that model.
    pub fn add_download_info_to_model(&self, model: &Model) -> ModelWithDownloadInfo {
        let files = model
            .files
            .iter()
            .map(|file| {
                let download = self
                    .downloads
                    .pending_downloads
                    .iter()
                    .find(|d| d.file.id == file.id)
                    .cloned();

                FileWithDownloadInfo {
                    file: file.clone(),
                    download,
                }
            })
            .collect();

        ModelWithDownloadInfo {
            model_id: model.id.clone(),
            name: model.name.clone(),
            summary: model.summary.clone(),
            size: model.size.clone(),
            requires: model.requires.clone(),
            architecture: model.architecture.clone(),
            like_count: model.like_count,
            download_count: model.download_count,
            released_at: model.released_at,
            author: model.author.clone(),
            files,
        }
    }

    pub fn get_model_and_file_download(&self, file_id: &str) -> (Model, File) {
        if let Some(result) = self
            .downloads
            .get_model_and_file_for_pending_download(file_id)
        {
            result
        } else {
            self.search
                .get_model_and_file_from_search_results(file_id)
                .unwrap()
        }
    }

    pub fn delete_file(&mut self, file_id: FileID) -> Result<()> {
        self.downloads.delete_file(file_id.clone())?;
        self.search
            .update_downloaded_file_in_search_results(&file_id, false);

        SignalToUI::set_ui_signal();
        Ok(())
    }

    pub fn process_event_signal(&mut self) {
        self.update_downloads();
        self.update_chat_messages();
        self.update_search_results();
        self.update_load_model();
    }

    fn update_search_results(&mut self) {
        match self.search.process_results() {
            Ok(Some(models)) => {
                self.search.set_models(models);
            }
            Ok(None) => {
                // No results arrived, do nothing
            }
            Err(_) => {
                self.search.set_models(vec![]);
            }
        }
    }

    fn update_chat_messages(&mut self) {
        let Some(chat) = self.chats.get_current_chat() else {
            return;
        };
        chat.borrow_mut().update_messages();
    }

    fn update_downloads(&mut self) {
        let completed_download_ids = self.downloads.refresh_downloads_data();

        // For search results let's trust on our local cache, but updating
        // the downloaded state of the files
        for file_id in completed_download_ids {
            self.search
                .update_downloaded_file_in_search_results(&file_id, true);
        }
    }

    fn init_current_chat(&mut self) {
        if let Some(chat_id) = self.chats.get_last_selected_chat_id() {
            self.chats.set_current_chat(chat_id);
        } else {
            self.chats.create_empty_chat();
        }

        // If there is no load model, let's try to load the one from preferences
        if self.chats.loaded_model.is_none() {
            if let Some(ref file_id) = self.preferences.current_chat_model {
                if let Some(file) = self
                    .downloads
                    .downloaded_files
                    .iter()
                    .find(|d| d.file.id == *file_id)
                    .map(|d| d.file.clone())
                {
                    self.load_model(&file);
                }
            }
        }
    }

    pub fn delete_chat(&mut self, chat_id: ChatID) {
        self.chats.remove_chat(chat_id);

        // TODO Decide proper behavior when deleting the current chat
        // For now, we just create a new empty chat because we don't fully
        // support having no chat selected
        self.init_current_chat();
    }
}
