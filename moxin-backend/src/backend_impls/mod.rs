use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use chrono::Utc;
use moxin_protocol::{
    data::{DownloadedFile, FileID, Model, PendingDownload},
    open_ai::{ChatRequestData, ChatResponse},
    protocol::{
        Command, FileDownloadResponse, LoadModelOptions, LoadModelResponse, LocalServerConfig,
        LocalServerResponse,
    },
};

use crate::store::{self, ModelFileDownloader, RemoteModel};

mod api_server;
mod chat_ui;

#[derive(Clone, Debug)]
enum ModelManagementCommand {
    GetFeaturedModels(Sender<anyhow::Result<Vec<Model>>>),
    SearchModels(String, Sender<anyhow::Result<Vec<Model>>>),
    DownloadFile(FileID, Sender<anyhow::Result<FileDownloadResponse>>),
    PauseDownload(FileID, Sender<anyhow::Result<()>>),
    CancelDownload(FileID, Sender<anyhow::Result<()>>),
    GetCurrentDownloads(Sender<anyhow::Result<Vec<PendingDownload>>>),
    GetDownloadedFiles(Sender<anyhow::Result<Vec<DownloadedFile>>>),
    DeleteFile(FileID, Sender<anyhow::Result<()>>),
    ChangeModelsLocation(PathBuf),
}

#[derive(Clone, Debug)]
enum ModelInteractionCommand {
    LoadModel(
        FileID,
        LoadModelOptions,
        Sender<anyhow::Result<LoadModelResponse>>,
    ),
    EjectModel(Sender<anyhow::Result<()>>),
    Chat(ChatRequestData, Sender<anyhow::Result<ChatResponse>>),
    StopChatCompletion(Sender<anyhow::Result<()>>),
    // Command to start a local server to interact with chat models
    StartLocalServer(
        LocalServerConfig,
        Sender<anyhow::Result<LocalServerResponse>>,
    ),
    // Command to stop the local server
    StopLocalServer(Sender<anyhow::Result<()>>),
}

#[derive(Clone, Debug)]
enum BuiltInCommand {
    Model(ModelManagementCommand),
    Interaction(ModelInteractionCommand),
}

impl From<Command> for BuiltInCommand {
    fn from(value: Command) -> Self {
        match value {
            Command::GetFeaturedModels(tx) => {
                Self::Model(ModelManagementCommand::GetFeaturedModels(tx))
            }
            Command::SearchModels(request, tx) => {
                Self::Model(ModelManagementCommand::SearchModels(request, tx))
            }
            Command::DownloadFile(file_id, tx) => {
                Self::Model(ModelManagementCommand::DownloadFile(file_id, tx))
            }
            Command::PauseDownload(file_id, tx) => {
                Self::Model(ModelManagementCommand::PauseDownload(file_id, tx))
            }
            Command::CancelDownload(file_id, tx) => {
                Self::Model(ModelManagementCommand::CancelDownload(file_id, tx))
            }
            Command::DeleteFile(file_id, tx) => {
                Self::Model(ModelManagementCommand::DeleteFile(file_id, tx))
            }
            Command::GetCurrentDownloads(tx) => {
                Self::Model(ModelManagementCommand::GetCurrentDownloads(tx))
            }
            Command::GetDownloadedFiles(tx) => {
                Self::Model(ModelManagementCommand::GetDownloadedFiles(tx))
            }
            Command::LoadModel(file_id, options, tx) => {
                Self::Interaction(ModelInteractionCommand::LoadModel(file_id, options, tx))
            }
            Command::EjectModel(tx) => Self::Interaction(ModelInteractionCommand::EjectModel(tx)),
            Command::Chat(request, tx) => {
                Self::Interaction(ModelInteractionCommand::Chat(request, tx))
            }
            Command::StopChatCompletion(tx) => {
                Self::Interaction(ModelInteractionCommand::StopChatCompletion(tx))
            }
            Command::StartLocalServer(config, tx) => {
                Self::Interaction(ModelInteractionCommand::StartLocalServer(config, tx))
            }
            Command::StopLocalServer(tx) => {
                Self::Interaction(ModelInteractionCommand::StopLocalServer(tx))
            }
            Command::ChangeModelsDir(path) => {
                Self::Model(ModelManagementCommand::ChangeModelsLocation(path))
            }
        }
    }
}

#[test]
fn test_chat() {
    use moxin_protocol::open_ai::*;

    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::<chat_ui::ChatBotModel>::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::GetDownloadedFiles(tx);
    bk.send(cmd).unwrap();
    let files = rx.recv().unwrap();
    assert!(files.is_ok());
    let files = files.unwrap();
    let file = files.first().unwrap();
    println!("{} {}", &file.file.id, &file.model.name);

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::LoadModel(
        file.file.id.clone(),
        LoadModelOptions {
            prompt_template: None,
            gpu_layers: moxin_protocol::protocol::GPULayers::Max,
            use_mlock: false,
            n_batch: 512,
            n_ctx: 512,
            rope_freq_scale: 0.0,
            rope_freq_base: 0.0,
            context_overflow_policy: moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
        },
        tx,
    );
    bk.send(cmd).unwrap();
    let r = rx.recv();
    assert!(r.is_ok());
    assert!(r.unwrap().is_ok());

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::Chat(
        ChatRequestData {
            messages: vec![Message {
                content: "hello".to_string(),
                role: Role::User,
                name: None,
            }],
            model: "llama-2-7b-chat.Q5_K_M".to_string(),
            frequency_penalty: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            presence_penalty: None,
            seed: None,
            stop: None,
            stream: Some(false),
            temperature: None,
            top_p: None,
            n: None,
            logit_bias: None,
        },
        tx,
    );
    bk.send(cmd).unwrap();
    if let Ok(Ok(ChatResponse::ChatFinalResponseData(data))) = rx.recv() {
        println!("{:?}", data.choices[0].message);
    }

    let (tx, rx) = std::sync::mpsc::channel();

    bk.send(Command::EjectModel(tx)).unwrap();
    rx.recv().unwrap().unwrap();
}

#[test]
fn test_chat_stop() {
    use moxin_protocol::open_ai::*;

    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::<chat_ui::ChatBotModel>::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::GetDownloadedFiles(tx);
    bk.send(cmd).unwrap();
    let files = rx.recv().unwrap();
    assert!(files.is_ok());
    let files = files.unwrap();
    let file = files.first().unwrap();
    println!("{} {}", &file.file.id, &file.model.name);

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::LoadModel(
        file.file.id.clone(),
        LoadModelOptions {
            prompt_template: None,
            gpu_layers: moxin_protocol::protocol::GPULayers::Max,
            use_mlock: false,
            n_batch: 512,
            n_ctx: 512,
            rope_freq_scale: 0.0,
            rope_freq_base: 0.0,
            context_overflow_policy: moxin_protocol::protocol::ContextOverflowPolicy::StopAtLimit,
        },
        tx,
    );
    bk.send(cmd).unwrap();
    let r = rx.recv();
    assert!(r.is_ok());
    assert!(r.unwrap().is_ok());

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::Chat(
        ChatRequestData {
            messages: vec![Message {
                content: "hello".to_string(),
                role: Role::User,
                name: None,
            }],
            model: "llama-2-7b-chat.Q5_K_M".to_string(),
            frequency_penalty: None,
            logprobs: None,
            top_logprobs: None,
            max_tokens: None,
            presence_penalty: None,
            seed: None,
            stop: None,
            stream: Some(true),
            temperature: None,
            top_p: None,
            n: None,
            logit_bias: None,
        },
        tx,
    );
    bk.send(cmd).unwrap();

    let mut i = 0;
    while let Ok(Ok(ChatResponse::ChatResponseChunk(data))) = rx.recv() {
        i += 1;
        println!(
            "{:?} {:?}",
            data.choices[0].delta, data.choices[0].finish_reason
        );
        if i == 5 {
            let (tx, rx) = std::sync::mpsc::channel();
            let cmd = Command::StopChatCompletion(tx);
            bk.send(cmd).unwrap();
            rx.recv().unwrap().unwrap();
        }
        if matches!(data.choices[0].finish_reason, Some(StopReason::Stop)) {
            break;
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();

    bk.send(Command::EjectModel(tx)).unwrap();
    rx.recv().unwrap().unwrap();
}

#[test]
fn test_download_file() {
    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::<chat_ui::ChatBotModel>::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::SearchModels("llama".to_string(), tx);
    bk.send(cmd).unwrap();
    let models = rx.recv().unwrap();
    assert!(models.is_ok());
    let models = models.unwrap();
    println!("{models:?}");

    let file = models[0].files[0].clone();
    println!("download {file:?}");

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::DownloadFile(file.id, tx.clone());
    bk.send(cmd).unwrap();

    let file = models[0].files[1].clone();
    println!("download {file:?}");

    let cmd = Command::DownloadFile(file.id, tx);
    bk.send(cmd).unwrap();

    println!();

    while let Ok(r) = rx.recv() {
        match r {
            Ok(FileDownloadResponse::Progress(file_id, progress)) => {
                println!("{file_id} progress: {:.2}%", progress);
            }
            Ok(FileDownloadResponse::Completed(file)) => {
                println!("Completed {file:?}");
            }
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }
}

#[test]
fn test_get_download_file() {
    let home = std::env::var("HOME").unwrap();
    let bk = BackendImpl::<chat_ui::ChatBotModel>::build_command_sender(
        format!("{home}/ai/models"),
        format!("{home}/ai/models"),
        3,
    );

    let (tx, rx) = std::sync::mpsc::channel();
    let cmd = Command::GetDownloadedFiles(tx);
    let _ = bk.send(cmd);

    let files = rx.recv().unwrap();

    println!("{files:?}");
}

#[derive(Debug, Clone)]
pub enum DownloadControlCommand {
    Stop(FileID),
}

pub type ChatModelBackend = BackendImpl<chat_ui::ChatBotModel>;
pub type LlamaEdgeApiServerBackend = BackendImpl<api_server::LLamaEdgeApiServer>;

pub trait BackendModel: Sized {
    fn new_or_reload(
        async_rt: &tokio::runtime::Runtime,
        old_model: Option<Self>,
        file: store::download_files::DownloadedFile,
        options: LoadModelOptions,
        tx: Sender<anyhow::Result<LoadModelResponse>>,
    ) -> Self;
    fn chat(
        &self,
        async_rt: &tokio::runtime::Runtime,
        data: ChatRequestData,
        tx: Sender<anyhow::Result<ChatResponse>>,
    ) -> bool;
    fn stop_chat(&self, async_rt: &tokio::runtime::Runtime);
    fn stop(self, async_rt: &tokio::runtime::Runtime);
}

pub struct BackendImpl<Model: BackendModel> {
    sql_conn: Arc<Mutex<rusqlite::Connection>>,
    #[allow(unused)]
    app_data_dir: PathBuf,
    models_dir: PathBuf,
    pub rx: Receiver<Command>,
    download_tx: tokio::sync::mpsc::UnboundedSender<(
        store::models::Model,
        store::download_files::DownloadedFile,
        Sender<anyhow::Result<FileDownloadResponse>>,
    )>,
    model: Option<Model>,

    #[allow(unused)]
    async_rt: tokio::runtime::Runtime,
    control_tx: tokio::sync::broadcast::Sender<DownloadControlCommand>,
}

impl<Model: BackendModel + Send + 'static> BackendImpl<Model> {
    /// # Arguments
    /// * `app_data_dir` - The directory where application data should be stored.
    /// * `models_dir` - The directory where models should be downloaded.
    /// * `max_download_threads` - Maximum limit on simultaneous file downloads.
    pub fn build_command_sender<A: AsRef<Path>, M: AsRef<Path>>(
        app_data_dir: A,
        models_dir: M,
        max_download_threads: usize,
    ) -> Sender<Command> {
        let app_data_dir = app_data_dir.as_ref().to_path_buf();
        wasmedge_sdk::plugin::PluginManager::load(None).unwrap();
        std::fs::create_dir_all(&app_data_dir).unwrap_or_else(|_| {
            panic!(
                "Failed to create the Moxin app data directory at {:?}",
                app_data_dir
            )
        });
        let sql_conn = rusqlite::Connection::open(app_data_dir.join("data.sqlite")).unwrap();

        // TODO Reorganize these bunch of functions, needs a little more of thought
        let _ = store::models::create_table_models(&sql_conn).unwrap();
        let _ = store::download_files::create_table_download_files(&sql_conn).unwrap();

        let sql_conn = Arc::new(Mutex::new(sql_conn));

        let (tx, rx) = std::sync::mpsc::channel();

        let async_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let (control_tx, _control_rx) = tokio::sync::broadcast::channel(100);
        let (download_tx, download_rx) = tokio::sync::mpsc::unbounded_channel();

        {
            let client = reqwest::Client::new();
            let downloader =
                ModelFileDownloader::new(client, sql_conn.clone(), control_tx.clone(), 0.1);
            async_rt.spawn(ModelFileDownloader::run_loop(
                downloader,
                max_download_threads.max(3),
                download_rx,
            ));
        }

        let mut backend = Self {
            sql_conn,
            app_data_dir,
            models_dir: models_dir.as_ref().into(),
            rx,
            download_tx,
            model: None,
            async_rt,
            control_tx,
        };

        std::thread::spawn(move || {
            backend.run_loop();
        });
        tx
    }

    fn handle_command(&mut self, built_in_cmd: BuiltInCommand) {
        match built_in_cmd {
            BuiltInCommand::Model(file) => match file {
                ModelManagementCommand::GetFeaturedModels(tx) => {
                    let res = store::RemoteModel::get_featured_model(100, 0);
                    match res {
                        Ok(remote_model) => {
                            let sql_conn = self.sql_conn.lock().unwrap();
                            let models = RemoteModel::to_model(&remote_model, &sql_conn)
                                .map_err(|e| anyhow::anyhow!("get featured error: {e}"));

                            let _ = tx.send(models);
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("get featured models error: {e}")));
                        }
                    }
                }
                ModelManagementCommand::SearchModels(search_text, tx) => {
                    let res = store::RemoteModel::search(&search_text, 100, 0);
                    match res {
                        Ok(remote_model) => {
                            let sql_conn = self.sql_conn.lock().unwrap();
                            let models = RemoteModel::to_model(&remote_model, &sql_conn)
                                .map_err(|e| anyhow::anyhow!("search models error: {e}"));

                            let _ = tx.send(models);
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("search models error: {e}")));
                        }
                    }
                }
                ModelManagementCommand::DownloadFile(file_id, tx) => {
                    //search model from remote
                    let search_model_from_remote = || -> anyhow::Result<( crate::store::models::Model , crate::store::download_files::DownloadedFile)> {
                        let (model_id, file) = file_id
                            .split_once("#")
                            .ok_or_else(|| anyhow::anyhow!("Illegal file_id"))?;
                        let mut res = store::RemoteModel::search(&model_id, 10, 0)
                            .map_err(|e| anyhow::anyhow!("search models error: {e}"))?;
                        let remote_model = res
                            .pop()
                            .ok_or_else(|| anyhow::anyhow!("model not found"))?;

                        let remote_file = remote_model
                            .files
                            .into_iter()
                            .find(|f| f.name == file)
                            .ok_or_else(|| anyhow::anyhow!("file not found"))?;

                        let download_model = crate::store::models::Model {
                            id: Arc::new(remote_model.id),
                            name: remote_model.name,
                            summary: remote_model.summary,
                            size: remote_model.size,
                            requires: remote_model.requires,
                            architecture: remote_model.architecture,
                            released_at: remote_model.released_at,
                            prompt_template: remote_model.prompt_template.clone(),
                            reverse_prompt: remote_model.reverse_prompt.clone(),
                            author: Arc::new(crate::store::models::Author {
                                name: remote_model.author.name,
                                url: remote_model.author.url,
                                description: remote_model.author.description,
                            }),
                            like_count: remote_model.like_count,
                            download_count: remote_model.download_count,
                        };

                        let download_file = crate::store::download_files::DownloadedFile {
                            id: Arc::new(file_id.clone()),
                            model_id: model_id.to_string(),
                            name: file.to_string(),
                            size: remote_file.size,
                            quantization: remote_file.quantization,
                            prompt_template: remote_model.prompt_template,
                            reverse_prompt: remote_model.reverse_prompt,
                            downloaded: false,
                            file_size: 0,
                            download_dir: self.models_dir.to_string_lossy().to_string(),
                            downloaded_at: Utc::now(),
                            tags:remote_file.tags,
                            featured: false,
                            sha256: remote_file.sha256.unwrap_or_default(),
                        };

                        Ok((download_model,download_file))
                    };

                    match search_model_from_remote() {
                        Ok((model, file)) => {
                            let _ = self.download_tx.send((model, file, tx));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e));
                        }
                    }
                }

                ModelManagementCommand::PauseDownload(file_id, tx) => {
                    let _ = self.control_tx.send(DownloadControlCommand::Stop(file_id));
                    let _ = tx.send(Ok(()));
                }

                ModelManagementCommand::CancelDownload(file_id, tx) => {
                    let file_id_ = file_id.clone();
                    let _ = self.control_tx.send(DownloadControlCommand::Stop(file_id_));

                    {
                        let conn = self.sql_conn.lock().unwrap();
                        let _ = store::download_files::DownloadedFile::remove(&file_id, &conn);
                    }
                    let _ = store::remove_downloaded_file(
                        self.models_dir.to_string_lossy().to_string(),
                        file_id,
                    );

                    let _ = tx.send(Ok(()));
                }

                ModelManagementCommand::DeleteFile(file_id, tx) => {
                    {
                        let conn = self.sql_conn.lock().unwrap();
                        let _ = store::download_files::DownloadedFile::remove(&file_id, &conn);
                    }

                    let _ = store::remove_downloaded_file(
                        self.models_dir.to_string_lossy().to_string(),
                        file_id,
                    );
                    let _ = tx.send(Ok(()));
                }

                ModelManagementCommand::GetDownloadedFiles(tx) => {
                    let downloads = {
                        let conn = self.sql_conn.lock().unwrap();
                        store::get_all_download_file(&conn)
                            .map_err(|e| anyhow::anyhow!("get download file error: {e}"))
                    };

                    let _ = tx.send(downloads);
                }

                ModelManagementCommand::GetCurrentDownloads(tx) => {
                    let pending_downloads = {
                        let conn = self.sql_conn.lock().unwrap();
                        store::get_all_pending_downloads(&conn)
                            .map_err(|e| anyhow::anyhow!("get pending download file error: {e}"))
                    };
                    let _ = tx.send(pending_downloads);
                }

                ModelManagementCommand::ChangeModelsLocation(path) => self.update_models_dir(path),
            },
            BuiltInCommand::Interaction(model_cmd) => match model_cmd {
                ModelInteractionCommand::LoadModel(file_id, options, tx) => {
                    let conn = self.sql_conn.lock().unwrap();
                    let download_file =
                        store::download_files::DownloadedFile::get_by_id(&conn, &file_id);

                    match download_file {
                        Ok(file) => {
                            nn_preload_file(&file);
                            let old_model = self.model.take();

                            let model =
                                Model::new_or_reload(&self.async_rt, old_model, file, options, tx);
                            self.model = Some(model);
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!("Load model error: {e}")));
                        }
                    }
                }
                ModelInteractionCommand::EjectModel(tx) => {
                    if let Some(model) = self.model.take() {
                        model.stop(&self.async_rt);
                    }
                    let _ = tx.send(Ok(()));
                }
                ModelInteractionCommand::Chat(data, tx) => {
                    if let Some(model) = &self.model {
                        model.chat(&self.async_rt, data, tx);
                    } else {
                        let _ = tx.send(Err(anyhow::anyhow!("Model not loaded")));
                    }
                }
                ModelInteractionCommand::StopChatCompletion(tx) => {
                    self.model
                        .as_ref()
                        .map(|model| model.stop_chat(&self.async_rt));
                    let _ = tx.send(Ok(()));
                }
                ModelInteractionCommand::StartLocalServer(_, _) => todo!(),
                ModelInteractionCommand::StopLocalServer(_) => todo!(),
            },
        }
    }

    pub fn update_models_dir<M: AsRef<Path>>(&mut self, models_dir: M) {
        self.models_dir = models_dir.as_ref().to_path_buf();
    }

    fn run_loop(&mut self) {
        loop {
            if let Ok(cmd) = self.rx.recv() {
                self.handle_command(cmd.into());
            } else {
                break;
            }
        }

        log::debug!("BackendImpl stop");
    }
}

pub fn nn_preload_file(file: &store::download_files::DownloadedFile) {
    let file_path = Path::new(&file.download_dir)
        .join(&file.model_id)
        .join(&file.name);

    let preloads = wasmedge_sdk::plugin::NNPreload::new(
        file.name.clone(),
        wasmedge_sdk::plugin::GraphEncoding::GGML,
        wasmedge_sdk::plugin::ExecutionTarget::AUTO,
        &file_path,
    );
    wasmedge_sdk::plugin::PluginManager::nn_preload(vec![preloads]);
}
