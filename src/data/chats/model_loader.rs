use makepad_widgets::SignalToUI;
use moxin_backend::Backend;
use moxin_protocol::{
    data::File,
    protocol::{Command, LoadModelOptions, LoadModelResponse},
};
use std::{
    sync::{mpsc::channel, Arc, Mutex},
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

pub struct ModelLoader {
    status: Arc<Mutex<ModelLoaderStatus>>,
    file: Option<File>,
}

impl ModelLoader {
    pub fn new() -> Self {
        Self {
            status: Default::default(),
            file: None,
        }
    }

    pub fn load(&mut self, file: File, backend: &Backend) {
        if self.is_loading() {
            panic!("ModelLoader is already loading a model");
        }

        let file_id = file.id.clone();
        self.file = Some(file);
        *self.status.lock().unwrap() = ModelLoaderStatus::Loading;

        let (tx, rx) = channel();
        let cmd = Command::LoadModel(
            file_id,
            LoadModelOptions {
                prompt_template: None,
                gpu_layers: moxin_protocol::protocol::GPULayers::Max,
                use_mlock: false,
                rope_freq_scale: 0.0,
                rope_freq_base: 0.0,
                context_overflow_policy:
                    moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
            },
            tx,
        );
        backend.command_sender.send(cmd).unwrap();

        let status = self.status.clone();
        thread::spawn(move || {
            let response = rx.recv();
            let mut status_lock = status.lock().unwrap();

            if let Ok(response) = response {
                match response {
                    Ok(LoadModelResponse::Completed(_)) => {
                        *status_lock = ModelLoaderStatus::Loaded;
                    }
                    Ok(_) => {
                        let msg = "Error loading model: Unexpected response";
                        *status_lock = ModelLoaderStatus::Failed;
                        eprintln!("{}", msg);
                    }
                    Err(err) => {
                        eprintln!("Error loading model: {:?}", &err);
                        *status_lock = ModelLoaderStatus::Failed;
                    }
                }
            } else {
                eprintln!("Error loading model: Internal communication error");
                *status_lock = ModelLoaderStatus::Failed;
            }

            SignalToUI::set_ui_signal();
        });
    }

    pub fn file(&self) -> Option<&File> {
        self.file.as_ref()
    }

    pub fn read_status(&self, f: impl FnOnce(&ModelLoaderStatus)) {
        f(&*self.status.lock().unwrap());
    }

    pub fn is_loaded(&self) -> bool {
        matches!(*self.status.lock().unwrap(), ModelLoaderStatus::Loaded)
    }

    pub fn is_loading(&self) -> bool {
        matches!(*self.status.lock().unwrap(), ModelLoaderStatus::Loading)
    }

    pub fn is_failed(&self) -> bool {
        matches!(*self.status.lock().unwrap(), ModelLoaderStatus::Failed)
    }

    pub fn is_finished(&self) -> bool {
        self.is_loaded() || self.is_failed()
    }

    pub fn is_pending(&self) -> bool {
        !self.is_finished()
    }

    // TODO: Improve
    pub fn block_until_finished(&self) {
        while self.is_pending() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
