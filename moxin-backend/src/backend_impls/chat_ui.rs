#[derive(Debug)]
pub enum TokenError {
    EndOfSequence = 1,
    ContextFull,
    PromptTooLong,
    TooLarge,
    InvalidEncoding,
    Other,
}

impl Into<StopReason> for TokenError {
    fn into(self) -> StopReason {
        match self {
            TokenError::EndOfSequence => StopReason::Stop,
            TokenError::ContextFull => StopReason::Length,
            TokenError::PromptTooLong => StopReason::Length,
            TokenError::TooLarge => StopReason::Length,
            TokenError::InvalidEncoding => StopReason::Stop,
            TokenError::Other => StopReason::Stop,
        }
    }
}

use std::{
    collections::HashMap,
    io::Read,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
};

use moxin_protocol::{
    open_ai::{
        ChatRequestData, ChatResponse, ChatResponseChunkData, ChatResponseData, ChoiceData,
        ChunkChoiceData, MessageData, Role, StopReason, UsageData,
    },
    protocol::{LoadModelOptions, LoadModelResponse, LoadedModelInfo},
};
use wasmedge_sdk::{
    error::{CoreError, CoreExecutionError},
    wasi::WasiModule,
    CallingFrame, ImportObject, Instance, Module, Store, Vm, WasmValue,
};

use crate::store::download_files::DownloadedFile;

#[derive(Debug)]
pub struct ChatBotUi {
    pub current_req: std::io::Cursor<Vec<u8>>,
    pub request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
    request_id: uuid::Uuid,
    chat_completion_message: Option<Vec<u8>>,
    pub token_tx: Option<Sender<anyhow::Result<ChatResponse>>>,
    running_controller: Arc<AtomicBool>,
    pub load_model_state: Option<(
        DownloadedFile,
        LoadModelOptions,
        Sender<anyhow::Result<LoadModelResponse>>,
    )>,
}

impl ChatBotUi {
    pub fn new(
        request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
        running_controller: Arc<AtomicBool>,
        file: DownloadedFile,
        load_model: LoadModelOptions,
        tx: Sender<anyhow::Result<LoadModelResponse>>,
    ) -> Self {
        Self {
            request_rx,
            request_id: uuid::Uuid::new_v4(),
            token_tx: None,
            running_controller,
            current_req: std::io::Cursor::new(vec![]),
            load_model_state: Some((file, load_model, tx)),
            chat_completion_message: None,
        }
    }

    fn init_request(&mut self) -> Result<(), ()> {
        if let Ok((req, tx)) = self.request_rx.recv() {
            // Init current_req
            if !req.stream.unwrap_or_default() {
                self.chat_completion_message = Some(Vec::with_capacity(
                    (req.max_tokens.unwrap_or(512) * 8) as usize,
                ))
            }
            *self.current_req.get_mut() = serde_json::to_vec(&req).unwrap();
            self.current_req.set_position(0);
            self.request_id = uuid::Uuid::new_v4();
            self.token_tx = Some(tx);
            self.running_controller.store(true, Ordering::Release);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn read_data(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.current_req.read(buf)?;
        if n == 0 {
            self.current_req.get_mut().clear();
            self.current_req.set_position(0);
        }
        Ok(n)
    }

    fn send_completion_output(
        token_tx: &mut Sender<anyhow::Result<ChatResponse>>,
        id: String,
        stop_reason: StopReason,
        chat_completion_message: &mut Option<Vec<u8>>,
    ) -> bool {
        if let Some(chat_completion_message) = chat_completion_message.take() {
            let _ = token_tx.send(Ok(ChatResponse::ChatFinalResponseData(ChatResponseData {
                id,
                choices: vec![ChoiceData {
                    finish_reason: stop_reason,
                    index: 0,
                    message: MessageData {
                        content: String::from_utf8_lossy(&chat_completion_message).to_string(),
                        role: Role::Assistant,
                    },
                    logprobs: None,
                }],
                created: 0,
                model: String::new(),
                system_fingerprint: String::new(),
                usage: UsageData {
                    completion_tokens: 0,
                    prompt_tokens: 0,
                    total_tokens: 0,
                },
                object: "chat.completion".to_string(),
            })));
        } else {
            let _ = token_tx.send(Ok(ChatResponse::ChatResponseChunk(ChatResponseChunkData {
                id: String::new(),
                choices: vec![ChunkChoiceData {
                    finish_reason: Some(stop_reason),
                    index: 0,
                    delta: MessageData {
                        content: String::new(),
                        role: Role::Assistant,
                    },
                    logprobs: None,
                }],
                created: 0,
                model: String::new(),
                system_fingerprint: String::new(),
                object: "chat.completion.chunk".to_string(),
            })));
        };
        true
    }

    fn send_streamed_output(
        token_tx: &mut Sender<anyhow::Result<ChatResponse>>,
        id: String,
        token: &[u8],
    ) -> bool {
        let _ = token_tx.send(Ok(ChatResponse::ChatResponseChunk(ChatResponseChunkData {
            id,
            choices: vec![ChunkChoiceData {
                finish_reason: None,
                index: 0,
                delta: MessageData {
                    content: String::from_utf8_lossy(token).to_string(),
                    role: Role::Assistant,
                },
                logprobs: None,
            }],
            created: 0,
            model: String::new(),
            system_fingerprint: String::new(),
            object: "chat.completion.chunk".to_string(),
        })));
        true
    }

    fn send_output(&mut self, output: Result<&[u8], TokenError>) -> bool {
        let id = self.request_id.to_string();
        match (
            output,
            &mut self.chat_completion_message,
            &mut self.token_tx,
        ) {
            (Ok(token), Some(chat_completion_message), Some(_tx)) => {
                chat_completion_message.extend_from_slice(token);
                true
            }
            (Ok(token), None, Some(tx)) => Self::send_streamed_output(tx, id, token),
            (Err(token_error), chat_completion_message, Some(tx)) => {
                Self::send_completion_output(tx, id, token_error.into(), chat_completion_message)
            }
            (_, _, None) => false,
        }
    }
}

fn get_input(
    data: &mut ChatBotUi,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([buf_ptr, buf_size]) = args.get(0..2) {
        let buf_ptr = buf_ptr.to_i32() as usize;
        let buf_size = buf_size.to_i32() as usize;

        let buf = mem
            .mut_slice::<u8>(buf_ptr, buf_size)
            .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

        if data.current_req.get_ref().is_empty() {
            if let Some((file, _, tx)) = data.load_model_state.take() {
                let file_id = file.id.as_ref().clone();
                let model_id = file.model_id;
                let _ = tx.send(Ok(LoadModelResponse::Completed(LoadedModelInfo {
                    file_id,
                    model_id,
                    information: String::new(),
                })));
            }

            data.init_request().or(Err(CoreError::Common(
                wasmedge_sdk::error::CoreCommonError::Interrupted,
            )))?;
        }

        let n = data.read_data(buf).unwrap();

        Ok(vec![WasmValue::from_i32(n as i32)])
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
    }
}

fn push_token(
    data: &mut ChatBotUi,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    if !data.running_controller.load(Ordering::Acquire) {
        return Ok(vec![WasmValue::from_i32(-1)]);
    }

    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([buf_ptr, buf_size]) = args.get(0..2) {
        let buf_ptr = buf_ptr.to_i32() as usize;
        let buf_size = buf_size.to_i32() as usize;

        let r = if buf_ptr != 0 {
            let buf = mem
                .mut_slice::<u8>(buf_ptr, buf_size)
                .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

            data.send_output(Ok(buf))
        } else {
            data.send_output(Err(TokenError::EndOfSequence))
        };

        Ok(vec![WasmValue::from_i32(if r { 0 } else { -1 })])
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
    }
}

fn return_token_error(
    data: &mut ChatBotUi,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    if let Some(error_code) = args.get(0) {
        let error_code = error_code.to_i32();
        let token_err = match error_code {
            1 => TokenError::EndOfSequence,
            2 => TokenError::ContextFull,
            3 => TokenError::PromptTooLong,
            4 => TokenError::TooLarge,
            5 => TokenError::InvalidEncoding,
            _ => TokenError::Other,
        };

        data.send_output(Err(token_err));

        Ok(vec![])
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncTypeMismatch))
    }
}

pub fn module(data: ChatBotUi) -> wasmedge_sdk::WasmEdgeResult<ImportObject<ChatBotUi>> {
    let mut module_builder = wasmedge_sdk::ImportObjectBuilder::new("chat_ui", data)?;
    module_builder.with_func::<(i32, i32), i32>("get_input", get_input)?;
    module_builder.with_func::<(i32, i32), i32>("push_token", push_token)?;
    module_builder.with_func::<i32, ()>("return_token_error", return_token_error)?;

    Ok(module_builder.build())
}

fn create_wasi(
    file: &DownloadedFile,
    load_model: &LoadModelOptions,
) -> wasmedge_sdk::WasmEdgeResult<WasiModule> {
    let ctx_size = if load_model.n_ctx > 0 {
        Some(load_model.n_ctx.to_string())
    } else {
        None
    };

    let n_gpu_layers = match load_model.gpu_layers {
        moxin_protocol::protocol::GPULayers::Specific(n) => Some(n.to_string()),
        moxin_protocol::protocol::GPULayers::Max => None,
    };

    let batch_size = if load_model.n_batch > 0 {
        Some(load_model.n_batch.to_string())
    } else {
        None
    };

    let mut prompt_template = load_model.prompt_template.clone();
    if prompt_template.is_none() && !file.prompt_template.is_empty() {
        prompt_template = Some(file.prompt_template.clone());
    }

    let reverse_prompt = if file.reverse_prompt.is_empty() {
        None
    } else {
        Some(file.reverse_prompt.clone())
    };

    let module_alias = file.name.as_ref();

    let mut args = vec!["chat_ui.wasm", "-a", module_alias];

    macro_rules! add_args {
        ($flag:expr, $value:expr) => {
            if let Some(ref value) = $value {
                args.push($flag);
                args.push(value.as_str());
            }
        };
    }

    add_args!("-c", ctx_size);
    add_args!("-g", n_gpu_layers);
    add_args!("-b", batch_size);
    add_args!("-p", prompt_template);
    add_args!("-r", reverse_prompt);

    WasiModule::create(Some(args), None, None)
}

pub fn run_wasm_by_downloaded_file(
    wasm_module: Module,
    request_rx: Receiver<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
    model_running_controller: Arc<AtomicBool>,
    file: DownloadedFile,
    load_model: LoadModelOptions,
    tx: Sender<anyhow::Result<LoadModelResponse>>,
) {
    use wasmedge_sdk::vm::SyncInst;
    use wasmedge_sdk::AsInstance;

    let mut instances: HashMap<String, &mut (dyn SyncInst)> = HashMap::new();

    let mut wasi = create_wasi(&file, &load_model).unwrap();
    let mut chatui = module(ChatBotUi::new(
        request_rx,
        model_running_controller,
        file,
        load_model,
        tx,
    ))
    .unwrap();

    instances.insert(wasi.name().to_string(), wasi.as_mut());
    let mut wasi_nn = wasmedge_sdk::plugin::PluginManager::load_plugin_wasi_nn().unwrap();
    instances.insert(wasi_nn.name().unwrap(), &mut wasi_nn);
    instances.insert(chatui.name().unwrap(), &mut chatui);

    let store = Store::new(None, instances).unwrap();
    let mut vm = Vm::new(store);
    vm.register_module(None, wasm_module.clone()).unwrap();

    let _ = vm.run_func(None, "_start", []);

    log::debug!("wasm exit");
}

pub struct Model {
    pub model_tx: Sender<(ChatRequestData, Sender<anyhow::Result<ChatResponse>>)>,
    pub model_running_controller: Arc<AtomicBool>,
    pub model_thread: JoinHandle<()>,
}

impl Model {
    pub fn new_or_reload(
        old_model: Option<Self>,
        wasm_module: Module,
        file: DownloadedFile,
        options: LoadModelOptions,
        tx: Sender<anyhow::Result<LoadModelResponse>>,
    ) -> Self {
        let (model_tx, request_rx) = std::sync::mpsc::channel();
        let model_running_controller = Arc::new(AtomicBool::new(false));
        let model_running_controller_ = model_running_controller.clone();

        let model_thread = std::thread::spawn(move || {
            run_wasm_by_downloaded_file(
                wasm_module,
                request_rx,
                model_running_controller_,
                file,
                options,
                tx,
            )
        });

        let new_model = Self {
            model_tx,
            model_thread,
            model_running_controller,
        };

        if let Some(old_model) = old_model {
            old_model.stop();
        }

        new_model
    }

    pub fn chat(&self, data: ChatRequestData, tx: Sender<anyhow::Result<ChatResponse>>) -> bool {
        self.model_tx.send((data, tx)).is_ok()
    }

    pub fn stop_chat(&self) {
        self.model_running_controller
            .store(false, Ordering::Release);
    }

    pub fn stop(self) {
        let Self {
            model_tx,
            model_thread,
            ..
        } = self;
        drop(model_tx);
        let _ = model_thread.join();
    }
}
