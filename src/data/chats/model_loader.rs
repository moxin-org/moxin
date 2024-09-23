use anyhow::anyhow;
use makepad_widgets::SignalToUI;
use moxin_protocol::{
    data::FileID,
    protocol::{Command, LoadModelOptions, LoadModelResponse},
};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    }, task::Context, thread
};

use super::chat::ContextWindow;

/// All posible states in which the loader can be.
#[derive(Debug, Default, Clone)]
pub enum ModelLoaderStatus {
    #[default]
    Unloaded,
    Loading,
    Loaded,
    Failed,
}

#[derive(Default)]
struct ModelLoaderInner {
    status: ModelLoaderStatus,
    file_id: Option<FileID>,
    context_window: ContextWindow,
}

/// Unit for handling the non-blocking loading of models across threads.
#[derive(Clone, Default)]
pub struct ModelLoader(Arc<Mutex<ModelLoaderInner>>);

impl ModelLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(
        &mut self,
        file_id: FileID,
        context_window: ContextWindow,
        command_sender: Sender<Command>,
    ) -> Result<(), anyhow::Error> {
        match self.status() {
            ModelLoaderStatus::Loading => {
                return Err(anyhow!("ModelLoader is already loading a model"));
            }
            ModelLoaderStatus::Loaded => {
                if let Some(prev_file_id) = self.file_id() {
                    if prev_file_id == file_id && context_window == self.context_window() {
                        return Ok(());
                    }
                }
            }
            _ => {}
        };

        self.set_status(ModelLoaderStatus::Loading);
        self.set_file_id(Some(file_id.clone()));
        self.set_context_window(context_window);

        let response = dispatch_load_command(command_sender, file_id.clone(), context_window).recv();

        let result = if let Ok(response) = response {
            match response {
                Ok(LoadModelResponse::Completed(_)) => {
                    self.set_status(ModelLoaderStatus::Loaded);
                    Ok(())
                }
                Ok(response) => {
                    self.set_status(ModelLoaderStatus::Failed);
                    Err(anyhow!("Unexpected response: {:?}", response))
                }
                Err(err) => {
                    self.set_status(ModelLoaderStatus::Failed);
                    Err(anyhow!(err))
                }
            }
        } else {
            self.set_status(ModelLoaderStatus::Failed);
            Err(anyhow!("Internal communication error"))
        };

        SignalToUI::set_ui_signal();
        result
    }

    pub fn load_async(&mut self, file_id: FileID, context_window: ContextWindow, command_sender: Sender<Command>) {
        let mut self_clone = self.clone();
        thread::spawn(move || {
            if let Err(err) = self_clone.load(file_id, context_window, command_sender) {
                eprintln!("Error loading model: {}", err);
            }
        });
    }

    fn set_status(&mut self, status: ModelLoaderStatus) {
        self.0.lock().unwrap().status = status;
    }

    fn set_file_id(&mut self, file_id: Option<FileID>) {
        self.0.lock().unwrap().file_id = file_id;
    }

    fn set_context_window(&mut self, context_window: ContextWindow) {
        self.0.lock().unwrap().context_window = context_window;
    }

    pub fn file_id(&self) -> Option<FileID> {
        self.0.lock().unwrap().file_id.clone()
    }

    pub fn context_window(&self) -> ContextWindow {
        self.0.lock().unwrap().context_window
    }

    pub fn status(&self) -> ModelLoaderStatus {
        self.0.lock().unwrap().status.clone()
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Loaded)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Loading)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status(), ModelLoaderStatus::Failed)
    }

    pub fn is_finished(&self) -> bool {
        self.is_loaded() || self.is_failed()
    }

    pub fn is_pending(&self) -> bool {
        !self.is_finished()
    }

    /// Get the file id of the model that is currently being loaded.
    /// Returns `None` if the model loader is not at a loading state.
    pub fn get_loading_file_id(&self) -> Option<FileID> {
        if self.is_loading() {
            return self.file_id();
        }

        None
    }
}

fn dispatch_load_command(
    command_sender: Sender<Command>,
    file_id: String,
    context_window: ContextWindow,
) -> Receiver<Result<LoadModelResponse, anyhow::Error>> {
    let (tx, rx) = channel();
    let cmd = Command::LoadModel(
        file_id,
        LoadModelOptions {
            prompt_template: None,
            gpu_layers: moxin_protocol::protocol::GPULayers::Max,
            use_mlock: false,
            rope_freq_scale: 0.0,
            rope_freq_base: 0.0,
            context_overflow_policy: moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            n_batch: None,
            n_ctx: context_window.into(),
        },
        tx,
    );
    command_sender.send(cmd).unwrap();
    rx
}
