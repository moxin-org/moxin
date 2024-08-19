use dora_node_api::{
    self,
    arrow::array::StringArray,
    dora_core::config::{DataId, NodeId},
    DoraNode, Event, MetadataParameters,
};
use eyre::Context;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, channel};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseQuestioner {
    pub task: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseWebSearch {
    pub task: String,
    pub result: HashMap<String, Value>,
}

#[derive(Debug, Copy, Clone)]
pub enum MaeAgent {
    Questioner,
    WebSearch
}

impl MaeAgent {
    pub fn name(&self) -> String {
        match self {
            MaeAgent::Questioner => "Questioner".to_string(),
            MaeAgent::WebSearch => "WebSearch".to_string()
        }
    }

    pub fn definition_file(&self) -> String {
        match self {
            MaeAgent::Questioner => "reasoner_agent.yml".to_string(),
            MaeAgent::WebSearch => "web_search_by_dspy.yml".to_string()
        }
    }

    pub fn parse_response(&self, response: String) -> MaeAgentResponse {
        match self {
            MaeAgent::Questioner => {
                let response = serde_json::from_str::<MaeResponseQuestioner>(&response).unwrap();
                MaeAgentResponse::QuestionerResponse(response)
            }
            MaeAgent::WebSearch => {
                let response = serde_json::from_str::<MaeResponseWebSearch>(&response).unwrap();
                MaeAgentResponse::WebSearchResponse(response)
            }
        }
    }
}

#[derive(Debug)]
pub enum MaeAgentResponse {
    QuestionerResponse(MaeResponseQuestioner),
    WebSearchResponse(MaeResponseWebSearch),
}

pub enum MaeAgentCommand {
    SendTask(String, MaeAgent, mpsc::Sender<MaeAgentResponse>),
    // CancelTask,
}

pub struct MaeBackend {
    pub command_sender: mpsc::Sender<MaeAgentCommand>,
}

impl MaeBackend {
    pub fn available_agents() -> Vec<MaeAgent> {
        vec![MaeAgent::Questioner, MaeAgent::WebSearch]
    }

    pub fn new() -> Self {
        let (command_sender, command_receiver) = channel();
        let (tx, rx) = channel();
        let backend = Self { command_sender };

        std::thread::spawn(move || {
            Self::start_receiver_loop(tx);
        });

        std::thread::spawn(move || {
            Self::start_sender_loop(command_receiver, rx);
        });

        backend
    }

    pub fn start_receiver_loop(sender: mpsc::Sender<String>) {
        loop {
            if let Ok((_node, mut events)) =
                DoraNode::init_from_node_id(NodeId::from("reasoner_output_moxin".to_string()))
            {
                while let Some(event) = events.recv() {
                    match event {
                        Event::Input {
                            id,
                            metadata: _,
                            data,
                        } => match data.data_type() {
                            dora_node_api::arrow::datatypes::DataType::Utf8 => {
                                let received_string: &str = TryFrom::try_from(&data)
                                    .context("expected string message")
                                    .expect("expected string message");

                                // Send response to frontend
                                sender
                                    .send(received_string.to_string())
                                    .expect("failed to send command");
                            }
                            _other => {
                                println!("Received id: {}, data: {:#?}", id, data);
                            }
                        },
                        _other => {}
                    }
                }
                // Waiting for the daemon to update ending of the dataflow.
                std::thread::sleep(std::time::Duration::from_secs(3));
            } else {
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        }
    }

    pub fn start_sender_loop(
        command_receiver: mpsc::Receiver<MaeAgentCommand>,
        rx: mpsc::Receiver<String>,
    ) {
        if let Ok((mut node, _events)) =
            DoraNode::init_from_node_id(NodeId::from("reasoner_task_input".to_string()))
        {
            loop {
                match command_receiver.recv().unwrap() {
                    MaeAgentCommand::SendTask(task, agent, tx) => {
                        // TODO Improve how we send the task prompt and the agent file
                        let data = StringArray::from(vec![
                            task.trim().to_string(),
                            agent.definition_file()
                        ]);
        
                        let res = node.send_output(
                            DataId::from("reasoner_task".to_string()),
                            MetadataParameters::default(),
                            data,
                        );
                        dbg!(res.unwrap());

                        match rx.recv() {
                            Ok(response) => {
                                let parsed = agent.parse_response(response);
                                tx.send(parsed).unwrap();
                            }
                            Err(err) => {
                                println!("Error: {:?}", err);
                            }
                        }
                    }
                }
            }
        }
    }
}
