use anyhow::{anyhow, Result};
use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_mae::MaeAgentCommand::{self, SendTask};
use moxin_mae::{MaeAgent, MaeAgentResponse, MaeBackend};
use moxin_protocol::data::{File, FileID};
use moxin_protocol::open_ai::*;
use moxin_protocol::protocol::Command;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::data::filesystem::{read_from_file, write_to_file};

use super::model_loader::ModelLoader;

pub type ChatID = u128;

#[derive(Clone, Debug)]
pub enum ChatMessageAction {
    ModelAppendDelta(String),
    ModelStreamingDone,
    MaeAgentResult(String, MaeAgent),
    MaeAgentCancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatEntity {
    ModelFile(FileID),
    Agent(MaeAgent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: usize,
    pub role: Role,
    pub username: Option<String>,
    pub entity: Option<ChatEntity>,
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

#[derive(Debug, Default)]
enum ChatState {
    #[default]
    Idle,
    Receiving,
    // The boolean indicates if the last message should be removed.
    Cancelled(bool),
}

#[derive(Serialize, Deserialize)]
struct ChatData {
    id: ChatID,
    last_used_entity: Option<ChatEntity>,
    system_prompt: Option<String>,
    messages: Vec<ChatMessage>,
    title: String,
    #[serde(default)]
    title_state: TitleState,
    #[serde(default)]
    accessed_at: chrono::DateTime<chrono::Utc>,

    // Legacy field, it can be removed in the future.
    last_used_file_id: Option<FileID>,
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
    pub last_used_entity: Option<ChatEntity>,
    pub messages: Vec<ChatMessage>,
    pub messages_update_sender: Sender<ChatMessageAction>,
    pub messages_update_receiver: Receiver<ChatMessageAction>,
    pub state: ChatState,
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
            last_used_entity: None,
            state: ChatState::Idle,
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

                // Fallback to last_used_file_id if last_used_entity is None.
                // Until this field is removed, we need to keep this logic.
                let chat_entity = data.last_used_entity.or_else(|| {
                    data.last_used_file_id
                        .map(|file_id| ChatEntity::ModelFile(file_id))
                });

                let chat = Chat {
                    id: data.id,
                    last_used_entity: chat_entity,
                    messages: data.messages,
                    title: data.title,
                    title_state: data.title_state,
                    state: ChatState::Idle,
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
            last_used_entity: self.last_used_entity.clone(),
            system_prompt: self.system_prompt.clone(),
            messages: self.messages.clone(),
            title: self.title.clone(),
            title_state: self.title_state,
            accessed_at: self.accessed_at,

            // Legacy field, it can be removed in the future.
            last_used_file_id: None,
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
            entity: None,
            content: prompt.clone(),
        });

        self.messages.push(ChatMessage {
            id: next_id + 1,
            role: Role::Assistant,
            username: Some(wanted_file.name.clone()),
            entity: Some(ChatEntity::ModelFile(wanted_file.id.clone())),
            content: "".to_string(),
        });

        self.state = ChatState::Receiving;
        self.last_used_entity = Some(ChatEntity::ModelFile(wanted_file.id.clone()));

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

                            let _ = store_chat_tx.send(ChatMessageAction::ModelAppendDelta(
                                data.choices[0].delta.content.clone(),
                            ));

                            if let Some(_reason) = &data.choices[0].finish_reason {
                                is_done = true;
                                let _ = store_chat_tx.send(ChatMessageAction::ModelStreamingDone);
                            }

                            SignalToUI::set_ui_signal();
                            if is_done {
                                break;
                            }
                        }
                        Ok(ChatResponse::ChatFinalResponseData(data)) => {
                            let _ = store_chat_tx.send(ChatMessageAction::ModelAppendDelta(
                                data.choices[0].message.content.clone(),
                            ));
                            let _ = store_chat_tx.send(ChatMessageAction::ModelStreamingDone);
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
        self.save();
    }

    pub fn send_message_to_agent(
        &mut self,
        agent: MaeAgent,
        prompt: String,
        mae_backend: &MaeBackend,
    ) {
        let (tx, rx) = mpsc::channel();
        mae_backend
            .command_sender
            .send(SendTask(prompt.clone(), agent.clone(), tx.clone()))
            .expect("failed to send message to agent");

        self.messages.push(ChatMessage {
            id: self.messages.len() + 1,
            role: Role::User,
            username: None,
            entity: None,
            content: prompt,
        });

        self.messages.push(ChatMessage {
            id: self.messages.len() + 1,
            role: Role::Assistant,
            username: Some(agent.name()),
            entity: Some(ChatEntity::Agent(agent.clone())),
            content: "".to_string(),
        });

        self.state = ChatState::Receiving;
        self.last_used_entity = Some(ChatEntity::Agent(agent.clone()));

        let store_chat_tx = self.messages_update_sender.clone();
        std::thread::spawn(move || {
            let mut had_some_update = false;
            '_loop: loop {
                match rx.recv() {
                    Ok(MaeAgentResponse::ResearchScholarUpdate(completed_step)) => {
                        println!(
                            "Received ResearchScholarUpdate from agent: {:?}",
                            completed_step
                        );
                        // TODO rename ModelAppendDelta if this is valid for models and agents
                        let _ = store_chat_tx.send(ChatMessageAction::ModelAppendDelta(
                            MaeAgentResponse::ResearchScholarUpdate(completed_step)
                                .to_text_messgae(),
                        ));

                        // Give some time between posting partial updates
                        had_some_update = true;
                        thread::sleep(Duration::from_millis(500));
                        SignalToUI::set_ui_signal();
                    }
                    Ok(response) => {
                        println!("Received response from agent: {:?}", response);
                        let _ = store_chat_tx.send(ChatMessageAction::MaeAgentResult(
                            response.to_text_messgae(),
                            agent.clone(),
                        ));

                        // Give some time to display the previos partial udpate if it existed
                        if had_some_update {
                            thread::sleep(Duration::from_millis(2000));
                        }

                        SignalToUI::set_ui_signal();
                        break '_loop;
                    }
                    Err(e) => {
                        println!("Error receiving response from agent: {:?}", e);
                        let _ = store_chat_tx.send(ChatMessageAction::MaeAgentCancelled);

                        SignalToUI::set_ui_signal();
                        break '_loop;
                    }
                }
            }
        });

        self.update_title_based_on_first_message();
        self.save();
    }

    pub fn cancel_streaming(&mut self, backend: &Backend) {
        if matches!(self.state, ChatState::Idle | ChatState::Cancelled(_)) {
            return;
        }

        let (tx, _rx) = channel();
        let cmd = Command::StopChatCompletion(tx);
        backend.command_sender.send(cmd).unwrap();

        let message = self.messages.last_mut().unwrap();
        if message.content.trim().is_empty() {
            self.state = ChatState::Cancelled(true);
            message.content = "Cancelling, please wait...".to_string();
        } else {
            self.state = ChatState::Cancelled(false);
        }
    }

    pub fn cancel_agent_interaction(&mut self, mae_backend: &MaeBackend) {
        if matches!(self.state, ChatState::Idle | ChatState::Cancelled(_)) {
            return;
        }

        let cmd = MaeAgentCommand::CancelTask;
        mae_backend.command_sender.send(cmd).unwrap();

        self.state = ChatState::Cancelled(true);
        let message = self.messages.last_mut().unwrap();
        message.content = "Cancelling, please wait...".to_string();
    }

    pub fn update_messages(&mut self) {
        for msg in self.messages_update_receiver.try_iter() {
            match msg {
                ChatMessageAction::ModelAppendDelta(response) => {
                    let last = self.messages.last_mut().unwrap();
                    last.content.push_str(&response);
                }
                ChatMessageAction::ModelStreamingDone => {
                    if matches!(self.state, ChatState::Cancelled(true)) {
                        // Remove the last message sent by the user if it was cancelled before getting any response
                        self.messages.pop();
                    }
                    self.state = ChatState::Idle;
                }
                ChatMessageAction::MaeAgentResult(response, _agent) => {
                    let last = self.messages.last_mut().unwrap();
                    last.content = response;
                    self.state = ChatState::Idle;
                }
                ChatMessageAction::MaeAgentCancelled => {
                    self.state = ChatState::Idle;
                    // Remove the last message sent by the user
                    self.messages.pop();
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

    pub fn is_receiving(&self) -> bool {
        matches!(self.state, ChatState::Receiving)
    }

    pub fn was_cancelled(&self) -> bool {
        matches!(self.state, ChatState::Cancelled(_))
    }
}

trait MaeAgentResponseFormatter {
    fn to_text_messgae(&self) -> String;
}

impl MaeAgentResponseFormatter for MaeAgentResponse {
    fn to_text_messgae(&self) -> String {
        match self {
            MaeAgentResponse::ReasonerResponse(response) => response.result.clone(),
            MaeAgentResponse::ResearchScholarResponse(response) => response.suggestion.clone(),
            MaeAgentResponse::SearchAssistantResponse(response) => {
                let mut formatted = format!("{}\n\n", response.result.web_search_results);

                let resouces_list = response
                    .result
                    .web_search_resource
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        format!(
                            "{}- [{}]({})\n",
                            i + 1,
                            r.name.replace("[", "\\[").replace("]", "\\]"),
                            r.url
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n\n");

                formatted.push_str(&resouces_list);
                formatted
            }
            MaeAgentResponse::ResearchScholarUpdate(completed_step) => {
                match completed_step.as_str() {
                    "keywords" => {
                        format!("Keywords were extracted from the task input.\n\nSearching and downloading papers (it could take some time)...\n\n")
                    }
                    "papers_info" => {
                        format!("Papers were downloaded and their information was extracted.\n\nAnalyzing papers...\n\n")
                    }
                    "paper_analyze_result" => {
                        format!("Papers were analyzed. Writing report...\n\n")
                    }
                    "writer_report" => {
                        format!("Wrapping up...\n\n")
                    }
                    _ => {
                        format!("")
                    }
                }
            }
        }
    }
}
