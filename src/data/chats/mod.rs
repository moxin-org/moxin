pub mod chat;
pub mod model_loader;

use anyhow::{Context, Result};
use chat::{Chat, ChatID};
use model_loader::ModelLoader;
use moxin_backend::Backend;
use moxin_protocol::data::*;
use moxin_protocol::protocol::Command;
use std::fs;
use std::sync::mpsc::channel;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use super::filesystem::setup_chats_folder;

pub struct Chats {
    pub backend: Rc<Backend>,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub loaded_model: Option<File>,
    pub model_loader: ModelLoader,

    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,
}

/// Posible states in which a model can be at runtime.
pub enum ModelStatus {
    Unloaded,
    Loading,
    Loaded,
    Failed,
}

impl Chats {
    pub fn new(backend: Rc<Backend>) -> Self {
        Self {
            backend,
            saved_chats: Vec::new(),
            current_chat_id: None,
            loaded_model: None,
            model_loader: ModelLoader::new(),
            chats_dir: setup_chats_folder(),
        }
    }

    /// Obtain the loading status for the model asigned to the current chat.
    /// If there is no chat selected, or no model assigned to the chat, it will return `None`.
    pub fn current_chat_model_loading_status(&self) -> Option<ModelStatus> {
        let current_chat = self.get_current_chat()?.borrow();
        let current_chat_model_id = current_chat.last_used_file_id.as_ref()?;

        let loading_model = self.get_currently_loading_model();
        let loaded_model = self.loaded_model.as_ref();

        if let Some(loading_model) = loading_model {
            if loading_model.id == *current_chat_model_id {
                return Some(ModelStatus::Loading);
            }
        }

        if let Some(loaded_model) = loaded_model {
            if loaded_model.id == *current_chat_model_id {
                return Some(ModelStatus::Loaded);
            }
        }

        Some(ModelStatus::Unloaded)
    }

    pub fn load_chats(&mut self) {
        let paths = fs::read_dir(&self.chats_dir).unwrap();

        for path in paths.map(|p| p.unwrap().path()) {
            let loaded_chat_result = Chat::load(path, self.chats_dir.clone());
            match loaded_chat_result {
                Err(e) => {
                    eprintln!("{}", &e.to_string());
                }
                Ok(loaded_chat) => self.saved_chats.push(RefCell::new(loaded_chat)),
            }
        }
    }

    pub fn get_last_selected_chat_id(&self) -> Option<ChatID> {
        self.saved_chats
            .iter()
            .max_by_key(|c| c.borrow().accessed_at)
            .map(|c| c.borrow().id)
    }

    pub fn load_model(&mut self, file: File) {
        if self.model_loader.is_loading() {
            return;
        }
        self.model_loader.load(file, self.backend.as_ref());
    }

    pub fn get_currently_loading_model(&self) -> Option<File> {
        if self.model_loader.is_loading() {
            return self.model_loader.file();
        }

        None
    }

    pub fn update_load_model(&mut self) {
        if self.model_loader.is_loaded() {
            self.loaded_model = self.model_loader.file();
        }
    }

    pub fn get_current_chat_id(&self) -> Option<ChatID> {
        self.current_chat_id
    }

    pub fn get_current_chat(&self) -> Option<&RefCell<Chat>> {
        if let Some(current_chat_id) = self.current_chat_id {
            self.saved_chats
                .iter()
                .find(|c| c.borrow().id == current_chat_id)
        } else {
            None
        }
    }

    pub fn get_chat_by_id(&self, chat_id: ChatID) -> Option<&RefCell<Chat>> {
        self.saved_chats.iter().find(|c| c.borrow().id == chat_id)
    }

    pub fn set_current_chat(&mut self, chat_id: ChatID) {
        self.current_chat_id = Some(chat_id);

        let mut chat = self.get_current_chat().unwrap().borrow_mut();
        chat.update_accessed_at();
        chat.save();
    }

    pub fn send_chat_message(&mut self, prompt: String) {
        let Some(loaded_model) = self.loaded_model.as_ref() else {
            println!("Skip sending message because loaded model not found");
            return;
        };

        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut()
                .send_message_to_model(prompt, loaded_model, self.backend.as_ref());
            chat.borrow().save();
        }
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().cancel_streaming(self.backend.as_ref());
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().delete_message(message_id);
            chat.borrow().save();
        }
    }

    pub fn edit_chat_message(
        &mut self,
        message_id: usize,
        updated_message: String,
        regenerate: bool,
    ) {
        if let Some(chat) = &mut self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            if regenerate {
                if let Some(loaded_model) = self.loaded_model.as_ref() {
                    if chat.is_streaming {
                        chat.cancel_streaming(self.backend.as_ref());
                    }

                    chat.remove_messages_from(message_id);
                    chat.send_message_to_model(
                        updated_message,
                        loaded_model,
                        self.backend.as_ref(),
                    );
                }
            } else {
                chat.edit_message(message_id, updated_message);
            }
            chat.save();
        }
    }

    pub fn eject_model(&mut self) -> Result<()> {
        let (tx, rx) = channel();
        self.backend
            .as_ref()
            .command_sender
            .send(Command::EjectModel(tx))
            .context("Failed to send eject model command")?;

        let _ = rx
            .recv()
            .context("Failed to receive eject model response")?
            .context("Eject model operation failed");

        self.loaded_model = None;
        Ok(())
    }

    pub fn create_empty_chat(&mut self) {
        let new_chat = RefCell::new(Chat::new(self.chats_dir.clone()));

        new_chat.borrow().save();

        self.current_chat_id = Some(new_chat.borrow().id);
        self.saved_chats.push(new_chat);
    }

    pub fn create_empty_chat_and_load_file(&mut self, file: File) {
        let new_chat = RefCell::new(Chat::new(self.chats_dir.clone()));
        new_chat.borrow().save();

        self.current_chat_id = Some(new_chat.borrow().id);
        self.saved_chats.push(new_chat);

        if self
            .loaded_model
            .as_ref()
            .map_or(true, |m| *m.id != file.id)
        {
            let _ = self.load_model(file);
        }
    }

    pub fn remove_chat(&mut self, chat_id: ChatID) {
        if let Some(chat) = self.saved_chats.iter().find(|c| c.borrow().id == chat_id) {
            chat.borrow().remove_saved_file();
        };
        self.saved_chats.retain(|c| c.borrow().id != chat_id);

        if let Some(current_chat_id) = self.current_chat_id {
            if current_chat_id == chat_id {
                self.current_chat_id = self.get_last_selected_chat_id();
            }
        }
    }
}
