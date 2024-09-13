pub mod chat;
pub mod model_loader;

use anyhow::{Context, Result};
use chat::{Chat, ChatEntity, ChatID};
use model_loader::ModelLoader;
use moxin_backend::Backend;
use moxin_mae::{MaeAgent, MaeBackend};
use moxin_protocol::data::*;
use moxin_protocol::protocol::Command;
use std::fs;
use std::sync::mpsc::channel;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use super::filesystem::setup_chats_folder;

pub struct Chats {
    pub backend: Rc<Backend>,
    pub mae_backend: Rc<MaeBackend>,
    pub saved_chats: Vec<RefCell<Chat>>,

    pub loaded_model: Option<File>,
    pub model_loader: ModelLoader,

    current_chat_id: Option<ChatID>,
    chats_dir: PathBuf,
}

impl Chats {
    pub fn new(backend: Rc<Backend>, mae_backend: Rc<MaeBackend>) -> Self {
        Self {
            backend,
            mae_backend,
            saved_chats: Vec::new(),
            current_chat_id: None,
            loaded_model: None,
            model_loader: ModelLoader::new(),
            chats_dir: setup_chats_folder(),
        }
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

    pub fn load_model(&mut self, file: &File) {
        self.cancel_chat_streaming();

        if let Some(mut chat) = self.get_current_chat().map(|c| c.borrow_mut()) {
            let new_file_id = Some(ChatEntity::ModelFile(file.id.clone()));

            if chat.associated_entity != new_file_id {
                chat.associated_entity = new_file_id;
                chat.save();
            }
        }

        if self.model_loader.is_loading() {
            return;
        }

        self.model_loader
            .load_async(file.id.clone(), self.backend.command_sender.clone());
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
        self.cancel_chat_streaming();
        self.current_chat_id = Some(chat_id);

        let mut chat = self.get_current_chat().unwrap().borrow_mut();
        chat.update_accessed_at();
        chat.save();
    }

    pub fn cancel_chat_streaming(&mut self) {
        if let Some(chat) = self.get_current_chat() {
            let mut chat = chat.borrow_mut();
            match chat.associated_entity {
                Some(ChatEntity::ModelFile(_)) => {
                    chat.cancel_streaming(self.backend.as_ref());
                }
                Some(ChatEntity::Agent(_)) => {
                    chat.cancel_agent_interaction(self.mae_backend.as_ref());
                }
                _ => {}
            }
        }
    }

    pub fn delete_chat_message(&mut self, message_id: usize) {
        if let Some(chat) = self.get_current_chat() {
            chat.borrow_mut().delete_message(message_id);
            chat.borrow().save();
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

    /// Get the file id to use with this chat, or the loaded file id as a fallback.
    /// The fallback is used if the chat does not have a file id set, or, if it has
    /// one but references a no longer existing (deleted) file.
    pub fn get_chat_file_id(&self, chat: &mut Chat) -> Option<FileID> {
        match &chat.associated_entity {
            Some(ChatEntity::ModelFile(file_id)) => {
                Some(file_id.clone())
            }
            _ => {
                let file_id = self.loaded_model.as_ref().map(|m| m.id.clone())?;
                Some(file_id)
            }
        }
    }

    pub fn create_empty_chat(&mut self) {
        let new_chat = RefCell::new(Chat::new(self.chats_dir.clone()));

        new_chat.borrow().save();

        self.current_chat_id = Some(new_chat.borrow().id);
        self.saved_chats.push(new_chat);
    }

    pub fn create_empty_chat_with_agent(&mut self, agent: MaeAgent) {
        self.create_empty_chat();
        if let Some(mut chat) = self.get_current_chat().map(|c| c.borrow_mut()) {
            chat.associated_entity = Some(ChatEntity::Agent(agent));
            chat.save();
        }
    }

    pub fn create_empty_chat_and_load_file(&mut self, file: &File) {
        let mut new_chat = Chat::new(self.chats_dir.clone());
        new_chat.associated_entity = Some(ChatEntity::ModelFile(file.id.clone()));
        new_chat.save();

        self.current_chat_id = Some(new_chat.id);
        self.saved_chats.push(RefCell::new(new_chat));

        self.load_model(file);
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
