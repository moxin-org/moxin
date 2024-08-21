use dora_node_api::{
    self,
    arrow::array::StringArray,
    dora_core::config::{DataId, NodeId},
    DoraNode, Event, MetadataParameters,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::mpsc::{self, channel};

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseQuestioner {
    pub task: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseWebSearchResource {
    pub name: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseWebSearchResult {
    pub web_search_results: String,
    #[serde(deserialize_with = "parse_web_search_resource")]
    pub web_search_resource: Vec<MaeResponseWebSearchResource>,
}

fn parse_web_search_resource<'de, D>(
    deserializer: D,
) -> Result<Vec<MaeResponseWebSearchResource>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let resources: Vec<MaeResponseWebSearchResource> =
        serde_json::from_str(&s).map_err(serde::de::Error::custom)?;

    Ok(resources)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseWebSearch {
    pub task: String,
    pub result: MaeResponseWebSearchResult,
}

#[derive(Debug, Copy, Clone)]
pub enum MaeAgent {
    Questioner,
    WebSearch,
}

impl MaeAgent {
    pub fn name(&self) -> String {
        match self {
            MaeAgent::Questioner => "Questioner".to_string(),
            MaeAgent::WebSearch => "WebSearch".to_string(),
        }
    }

    pub fn definition_file(&self) -> String {
        match self {
            MaeAgent::Questioner => "reasoner_agent.yml".to_string(),
            MaeAgent::WebSearch => "web_search_by_dspy.yml".to_string(),
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
        let backend = Self { command_sender };

        std::thread::spawn(move || {
            Self::main_loop(command_receiver);
        });

        backend
    }

    pub fn main_loop(command_receiver: mpsc::Receiver<MaeAgentCommand>) {
        let Ok((_node, mut events)) =
            DoraNode::init_from_node_id(NodeId::from("reasoner_output_moxin".to_string()))
        else {
            eprintln!("Failed to initialize node: reasoner_output_moxin");
            return;
        };

        let Ok((mut node, _events)) =
            DoraNode::init_from_node_id(NodeId::from("reasoner_task_input".to_string()))
        else {
            eprintln!("Failed to initialize node: reasoner_task_input");
            return;
        };

        loop {
            let sender_to_frontend: mpsc::Sender<MaeAgentResponse>;
            let current_agent: MaeAgent;

            // Receive command from frontend
            match command_receiver.recv().unwrap() {
                MaeAgentCommand::SendTask(task, agent, tx) => {
                    // TODO Improve how we send the task prompt and the agent file
                    let data =
                        StringArray::from(vec![task.trim().to_string(), agent.definition_file()]);

                    node.send_output(
                        DataId::from("reasoner_task".to_string()),
                        MetadataParameters::default(),
                        data,
                    )
                    .expect("failed to send task to reasoner");

                    sender_to_frontend = tx;
                    current_agent = agent;
                }
            }

            // Listen for events from reasoner to send the response to frontend
            '_while: while let Some(event) = events.recv() {
                match event {
                    Event::Input {
                        id,
                        metadata: _,
                        data,
                    } => {
                        match data.data_type() {
                            dora_node_api::arrow::datatypes::DataType::Utf8 => {
                                let received_string: &str =
                                    TryFrom::try_from(&data).expect("expected string message");

                                let parsed =
                                    current_agent.parse_response(received_string.to_string());
                                sender_to_frontend
                                    .send(parsed)
                                    .expect("failed to send command");
                            }
                            _other => {
                                println!("Received id: {}, data: {:#?}", id, data);
                            }
                        }

                        // Stop listening for events after receiving the actual response
                        break '_while;
                    }
                    _other => {}
                }
            }
        }
    }

    // For testing purposes
    pub fn new_fake() -> Self {
        let (command_sender, command_receiver) = channel();
        let backend = Self { command_sender };

        std::thread::spawn(move || {
            loop {
                // Receive command from frontend
                match command_receiver.recv().unwrap() {
                    MaeAgentCommand::SendTask(task, agent, tx) => match agent {
                        MaeAgent::Questioner => {
                            let response = MaeResponseQuestioner {
                                task: task.clone(),
                                result: "This is a fake response".to_string(),
                            };
                            tx.send(MaeAgentResponse::QuestionerResponse(response))
                                .expect("failed to send command");
                        }
                        MaeAgent::WebSearch => {
                            let response = MaeResponseWebSearch {
                                task: task.clone(),
                                result: MaeResponseWebSearchResult {
                                    web_search_results: "This is a fake response".to_string(),
                                    web_search_resource: vec![MaeResponseWebSearchResource {
                                        name: "Fake resource".to_string(),
                                        url: "https://fake.com".to_string(),
                                        snippet: "This is a fake snippet".to_string(),
                                    }],
                                },
                            };
                            tx.send(MaeAgentResponse::WebSearchResponse(response))
                                .expect("failed to send command");
                        }
                    },
                }
            }
        });

        backend
    }
}
