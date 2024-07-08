mod backend_impls;
mod store;

use moxin_protocol::protocol::Command;
use std::{path::Path, sync::mpsc};

pub struct Backend {
    pub command_sender: mpsc::Sender<Command>,
}

impl Backend {
    /// # Arguments
    /// * `app_data_dir` - The directory where application data should be stored.
    /// * `models_dir` - The directory where models should be downloaded.
    /// * `max_download_threads` - Maximum limit on simultaneous file downloads.
    pub fn new<A: AsRef<Path>, M: AsRef<Path>>(
        app_data_dir: A,
        models_dir: M,
        max_download_threads: usize,
    ) -> Backend {
        let command_sender = backend_impls::ChatModelBackend::build_command_sender(
            app_data_dir,
            models_dir,
            max_download_threads,
        );
        Backend { command_sender }
    }
}
