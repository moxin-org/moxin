use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_protocol::{
    data::{File, FileID},
    protocol::{Command, LoadModelOptions, LoadModelResponse},
};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

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
        command_sender: Sender<Command>,
    ) -> Receiver<Result<(), ()>> {
        if self.is_loading() {
            panic!("ModelLoader is already loading a model");
        }

        let (load_tx, load_rx) = channel();

        if let Some(prev_file_id) = self.file_id() {
            if prev_file_id == file_id {
                let _ = load_tx.send(Ok(()));
                return load_rx;
            }
        }

        let mut outer_lock = self.0.lock().unwrap();
        outer_lock.file_id = Some(file_id.clone());
        outer_lock.status = ModelLoaderStatus::Loading;

        let rx = dispatch_load_command(command_sender, file_id.clone());
        let inner = self.0.clone();
        thread::spawn(move || {
            let response = rx.recv();
            let mut inner_lock = inner.lock().unwrap();

            if let Ok(response) = response {
                match response {
                    Ok(LoadModelResponse::Completed(_)) => {
                        inner_lock.status = ModelLoaderStatus::Loaded;
                    }
                    Ok(_) => {
                        let msg = "Error loading model: Unexpected response";
                        inner_lock.status = ModelLoaderStatus::Failed;
                        eprintln!("{}", msg);
                    }
                    Err(err) => {
                        eprintln!("Error loading model: {:?}", &err);
                        inner_lock.status = ModelLoaderStatus::Failed;
                    }
                }
            } else {
                eprintln!("Error loading model: Internal communication error");
                inner_lock.status = ModelLoaderStatus::Failed;
            }

            match inner_lock.status {
                ModelLoaderStatus::Loaded => {
                    let _ = load_tx.send(Ok(()));
                }
                ModelLoaderStatus::Failed => {
                    let _ = load_tx.send(Err(()));
                }
                _ => {
                    panic!("ModelLoader finished with unexpected status");
                }
            }

            SignalToUI::set_ui_signal();
        });

        load_rx
    }

    pub fn file_id(&self) -> Option<FileID> {
        self.0.lock().unwrap().file_id.clone()
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
        },
        tx,
    );
    command_sender.send(cmd).unwrap();
    rx
}
