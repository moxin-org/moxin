use anyhow::{anyhow, Result};
use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_mae::MaeAgent;
use moxin_protocol::data::{File, FileID};
use moxin_protocol::open_ai::*;
use moxin_protocol::protocol::Command;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::data::filesystem::{read_from_file, write_to_file};

use super::model_loader::ModelLoader;

pub type ChatID = u128;

#[derive(Clone, Debug)]
pub enum ChatTokenArrivalAction {
    AppendDelta(String),
    StreamingDone,

    MaeResult(String, MaeAgent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: usize,
    pub role: Role,
    pub username: Option<String>,
    pub content: String,
}

impl ChatMessage {
    pub fn is_assistant(&self) -> bool {
        matches!(self.role, Role::Assistant)
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
enum TitleState {
    #[default]
    Default,
    Updated,
}

#[derive(Serialize, Deserialize)]
struct ChatData {
    id: ChatID,
    last_used_file_id: Option<FileID>,
    system_prompt: Option<String>,
    messages: Vec<ChatMessage>,
    title: String,
    #[serde(default)]
    title_state: TitleState,
    #[serde(default)]
    accessed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct ChatInferenceParams {
    pub frequency_penalty: f32,
    pub max_tokens: u32,
    pub presence_penalty: f32,
    pub temperature: f32,
    pub top_p: f32,
    pub stream: bool,
    pub stop: String,
}

impl Default for ChatInferenceParams {
    fn default() -> Self {
        Self {
            frequency_penalty: 0.0,
            max_tokens: 2048,
            presence_penalty: 0.0,
            temperature: 1.0,
            top_p: 1.0,
            stream: true,
            stop: "".into(),
        }
    }
}

#[derive(Debug)]
pub struct Chat {
    /// Unix timestamp in ms.
    pub id: ChatID,
    pub last_used_file_id: Option<FileID>,
    pub messages: Vec<ChatMessage>,
    pub messages_update_sender: Sender<ChatTokenArrivalAction>,
    pub messages_update_receiver: Receiver<ChatTokenArrivalAction>,
    pub is_streaming: bool,
    pub inferences_params: ChatInferenceParams,
    pub system_prompt: Option<String>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,

    title: String,
    title_state: TitleState,

    chats_dir: PathBuf,
}

impl Chat {
    pub fn new(chats_dir: PathBuf) -> Self {
        let (tx, rx) = channel();

        // Get Unix timestamp in ms for id.
        let id = chrono::Utc::now().timestamp_millis() as u128;

        Self {
            id,
            title: String::from("New Chat"),
            messages: vec![],
            messages_update_sender: tx,
            messages_update_receiver: rx,
            last_used_file_id: None,
            is_streaming: false,
            title_state: TitleState::default(),
            chats_dir,
            inferences_params: ChatInferenceParams::default(),
            system_prompt: None,
            accessed_at: chrono::Utc::now(),
        }
    }

    pub fn load(path: PathBuf, chats_dir: PathBuf) -> Result<Self> {
        let (tx, rx) = channel();

        match read_from_file(path) {
            Ok(json) => {
                let data: ChatData = serde_json::from_str(&json)?;
                let chat = Chat {
                    id: data.id,
                    last_used_file_id: data.last_used_file_id,
                    messages: data.messages,
                    title: data.title,
                    title_state: data.title_state,
                    is_streaming: false,
                    messages_update_sender: tx,
                    messages_update_receiver: rx,
                    chats_dir,
                    inferences_params: ChatInferenceParams::default(),
                    system_prompt: data.system_prompt,
                    accessed_at: data.accessed_at,
                };
                Ok(chat)
            }
            Err(_) => Err(anyhow!("Couldn't read chat file from path")),
        }
    }

    pub fn save(&self) {
        let data = ChatData {
            id: self.id,
            last_used_file_id: self.last_used_file_id.clone(),
            system_prompt: self.system_prompt.clone(),
            messages: self.messages.clone(),
            title: self.title.clone(),
            title_state: self.title_state,
            accessed_at: self.accessed_at,
        };
        let json = serde_json::to_string(&data).unwrap();
        let path = self.chats_dir.join(self.file_name());
        write_to_file(path, &json).unwrap();
    }

    pub fn remove_saved_file(&self) {
        let path = self.chats_dir.join(self.file_name());
        std::fs::remove_file(path).unwrap();
    }

    fn file_name(&self) -> String {
        format!("{}.chat.json", self.id)
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.title_state = TitleState::Updated;
    }

    fn update_title_based_on_first_message(&mut self) {
        // If it hasnt been updated, and theres at least one message, use the first
        // one as title. Else we just return the default one.
        if matches!(self.title_state, TitleState::Default) {
            if let Some(message) = self.messages.first() {
                let max_char_length = 25;
                let ellipsis = "...";

                let title = if message.content.len() > max_char_length {
                    let mut truncated = message
                        .content
                        .chars()
                        .take(max_char_length)
                        .collect::<String>()
                        .replace('\n', " ");
                    truncated.push_str(ellipsis);
                    truncated
                } else {
                    message.content.clone()
                };

                self.set_title(title);
            }
        }
    }
    pub fn send_message_to_model(
        &mut self,
        prompt: String,
        wanted_file: &File,
        mut model_loader: ModelLoader,
        backend: &Backend,
    ) {
        let (tx, rx) = channel();
        let mut messages: Vec<_> = self
            .messages
            .iter()
            .map(|message| Message {
                content: message.content.clone(),
                role: message.role.clone(),
                name: None,
            })
            .collect();

        messages.push(Message {
            content: prompt.clone(),
            role: Role::User,
            name: None,
        });

        if let Some(system_prompt) = &self.system_prompt {
            messages.insert(
                0 as usize,
                Message {
                    content: system_prompt.clone(),
                    role: Role::System,
                    name: None,
                },
            );
        } else {
            messages.insert(
                0 as usize,
                Message {
                    content: "You are a helpful, respectful, and honest assistant.".to_string(),
                    role: Role::System,
                    name: None,
                },
            );
        }

        let ip = &self.inferences_params;
        let cmd = Command::Chat(
            ChatRequestData {
                messages,
                model: wanted_file.name.clone(),
                frequency_penalty: Some(ip.frequency_penalty),
                logprobs: None,
                top_logprobs: None,
                max_tokens: Some(ip.max_tokens),
                presence_penalty: Some(ip.presence_penalty),
                seed: None,
                stop: Some(
                    ip.stop
                        .split(",")
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect(),
                ),
                stream: Some(ip.stream),
                temperature: Some(ip.temperature),
                top_p: Some(ip.top_p),
                n: None,
                logit_bias: None,
            },
            tx,
        );

        let next_id = self.messages.last().map(|m| m.id).unwrap_or(0) + 1;
        self.messages.push(ChatMessage {
            id: next_id,
            role: Role::User,
            username: None,
            content: prompt.clone(),
        });

        self.messages.push(ChatMessage {
            id: next_id + 1,
            role: Role::Assistant,
            username: Some(wanted_file.name.clone()),
            content: "".to_string(),
        });

        self.is_streaming = true;

        let store_chat_tx = self.messages_update_sender.clone();
        let wanted_file = wanted_file.clone();
        let command_sender = backend.command_sender.clone();
        thread::spawn(move || {
            if let Err(err) = model_loader.load(wanted_file.id, command_sender.clone()) {
                eprintln!("Error loading model: {}", err);
                return;
            }

            command_sender.send(cmd).unwrap();

            loop {
                if let Ok(response) = rx.recv() {
                    match response {
                        Ok(ChatResponse::ChatResponseChunk(data)) => {
                            let mut is_done = false;

                            let _ = store_chat_tx.send(ChatTokenArrivalAction::AppendDelta(
                                data.choices[0].delta.content.clone(),
                            ));

                            if let Some(_reason) = &data.choices[0].finish_reason {
                                is_done = true;
                                let _ = store_chat_tx.send(ChatTokenArrivalAction::StreamingDone);
                            }

                            SignalToUI::set_ui_signal();
                            if is_done {
                                break;
                            }
                        }
                        Ok(ChatResponse::ChatFinalResponseData(data)) => {
                            let _ = store_chat_tx.send(ChatTokenArrivalAction::AppendDelta(
                                data.choices[0].message.content.clone(),
                            ));
                            let _ = store_chat_tx.send(ChatTokenArrivalAction::StreamingDone);
                            SignalToUI::set_ui_signal();
                            break;
                        }
                        Err(err) => eprintln!("Error receiving response chunk: {:?}", err),
                    }
                } else {
                    break;
                };
            }
        });

        self.update_title_based_on_first_message();
    }

    pub fn cancel_streaming(&mut self, backend: &Backend) {
        let (tx, _rx) = channel();
        let cmd = Command::StopChatCompletion(tx);
        backend.command_sender.send(cmd).unwrap();

        makepad_widgets::log!("Cancel streaming");
    }

    pub fn update_messages(&mut self) {
        for msg in self.messages_update_receiver.try_iter() {
            match msg {
                ChatTokenArrivalAction::AppendDelta(response) => {
                    let last = self.messages.last_mut().unwrap();
                    last.content.push_str(&response);
                }
                // Mae Prototype
                ChatTokenArrivalAction::MaeResult(response, agent) => {
                    match agent {
                        MaeAgent::Questioner => {
                            let response =
                                serde_json::from_str::<MaeResponseQuestioner>(&response).unwrap();

                            self.messages.push(ChatMessage {
                                id: self.messages.len() + 1,
                                role: Role::User,
                                username: None,
                                content: response.task,
                            });

                            self.messages.push(ChatMessage {
                                id: self.messages.len() + 1,
                                role: Role::Assistant,
                                username: Some("Reasoner Agent".to_string()),
                                content: response.result,
                            });
                        }
                        MaeAgent::WebSearch => {
                            let response =
                                serde_json::from_str::<MaeResponseWebSearch>(&response).unwrap();

                            self.messages.push(ChatMessage {
                                id: self.messages.len() + 1,
                                role: Role::User,
                                username: None,
                                content: response.task,
                            });

                            self.messages.push(ChatMessage {
                                id: self.messages.len() + 1,
                                role: Role::Assistant,
                                username: Some("Web Search Agent".to_string()),
                                content: response.result["web_search_results"].as_str().unwrap().to_string(),
                            });
                        }
                    }

                    dbg!(&self.messages);
                }

                ChatTokenArrivalAction::StreamingDone => {
                    self.is_streaming = false;
                }
            }
            self.save();
        }
    }

    pub fn delete_message(&mut self, message_id: usize) {
        self.messages.retain(|message| message.id != message_id);
    }

    pub fn edit_message(&mut self, message_id: usize, updated_message: String) {
        if let Some(message) = self.messages.iter_mut().find(|m| m.id == message_id) {
            message.content = updated_message;
        }
    }

    pub fn remove_messages_from(&mut self, message_id: usize) {
        let message_index = self
            .messages
            .iter()
            .position(|m| m.id == message_id)
            .unwrap();
        self.messages.truncate(message_index);
    }

    pub fn update_accessed_at(&mut self) {
        self.accessed_at = chrono::Utc::now();
    }
}

#[derive(Serialize, Deserialize)]
struct MaeResponseQuestioner {
    pub task: String,
    pub result: String,
}

#[derive(Serialize, Deserialize)]
struct MaeResponseWebSearch {
    pub task: String,
    pub result: HashMap<String, Value>,
}
