use dora_node_api::{
    self,
    arrow::{array::StringArray, datatypes},
    dora_core::config::{DataId, NodeId},
    DoraNode, Event, MetadataParameters,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashMap,
    sync::mpsc::{self, channel},
};

use dora_node_api::arrow::array::AsArray;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseQuestioner {
    pub task: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponsePapersResearch {
    pub task: String,
    pub suggestion: String,
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
    PapersResearch,
}

pub enum MaeAgentWorkflow {
    BasicReasoner(String),
    Paper,
}

impl MaeAgent {
    pub fn name(&self) -> String {
        match self {
            MaeAgent::Questioner => "Questioner".to_string(),
            MaeAgent::WebSearch => "WebSearch".to_string(),
            MaeAgent::PapersResearch => "PapersResearch".to_string(),
        }
    }

    pub fn workflow(&self) -> MaeAgentWorkflow {
        match self {
            MaeAgent::Questioner => {
                MaeAgentWorkflow::BasicReasoner("reasoner_agent.yml".to_string())
            }
            MaeAgent::WebSearch => {
                MaeAgentWorkflow::BasicReasoner("web_search_by_dspy.yml".to_string())
            }
            MaeAgent::PapersResearch => MaeAgentWorkflow::Paper,
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
            MaeAgent::PapersResearch => {
                let response =
                    serde_json::from_str::<MaeResponsePapersResearch>(&response).unwrap();
                MaeAgentResponse::PapersResearchResponse(response)
            }
        }
    }
}

#[derive(Debug)]
pub enum MaeAgentResponse {
    QuestionerResponse(MaeResponseQuestioner),
    WebSearchResponse(MaeResponseWebSearch),
    PapersResearchResponse(MaeResponsePapersResearch),

    // This is not a final response, it is an indication that the agent is still working
    // but some step was completed
    PapersResearchUpdate(String),
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

    pub fn new(options: HashMap<String, String>) -> Self {
        let (command_sender, command_receiver) = channel();
        let backend = Self { command_sender };

        std::thread::spawn(move || {
            Self::main_loop(command_receiver, options);
        });

        backend
    }

    pub fn main_loop(
        command_receiver: mpsc::Receiver<MaeAgentCommand>,
        options: HashMap<String, String>,
    ) {
        let Ok((mut node, _events)) =
            DoraNode::init_from_node_id(NodeId::from("reasoner_task_input".to_string()))
        else {
            eprintln!("Failed to initialize node: reasoner_task_input");
            return;
        };

        let Ok((mut paper_node, _events)) =
            DoraNode::init_from_node_id(NodeId::from("paper_task_input".to_string()))
        else {
            eprintln!("Failed to initialize node: paper_task_input");
            return;
        };

        let Ok((_node, mut events)) =
            DoraNode::init_from_node_id(NodeId::from("reasoner_output_moxin".to_string()))
        else {
            eprintln!("Failed to initialize node: reasoner_output_moxin");
            return;
        };

        let Ok((_node, mut paper_events)) =
            DoraNode::init_from_node_id(NodeId::from("paper_output_moxin".to_string()))
        else {
            eprintln!("Failed to initialize node: paper_output_moxin");
            return;
        };

        println!("MAE backend started");

        loop {
            let sender_to_frontend: mpsc::Sender<MaeAgentResponse>;
            let current_agent: MaeAgent;

            // Receive command from frontend
            match command_receiver.recv().unwrap() {
                MaeAgentCommand::SendTask(task, agent, tx) => {
                    match agent.workflow() {
                        MaeAgentWorkflow::BasicReasoner(definition_file) => {
                            // Information sent to MAE agent goes in the form of an array
                            // It contains:
                            // 1. The task to be performed (user prompt)
                            // 2. The definition file of the agent
                            // 3. A hash map of options encoded in JSON format
                            let data = StringArray::from(vec![
                                task.trim().to_string(),
                                definition_file,
                                serde_json::to_string(&options).unwrap(),
                            ]);

                            node.send_output(
                                DataId::from("reasoner_task".to_string()),
                                MetadataParameters::default(),
                                data,
                            )
                            .expect("failed to send task to reasoner");
                        }
                        MaeAgentWorkflow::Paper => {
                            // Information sent to MAE agent goes in the form of an array
                            // It contains:
                            // 1. The task to be performed (user prompt)
                            // 3. A hash map of options encoded in JSON format
                            let data = StringArray::from(vec![
                                task.trim().to_string(),
                                serde_json::to_string(&options).unwrap(),
                            ]);

                            paper_node
                                .send_output(
                                    DataId::from("task".to_string()),
                                    MetadataParameters::default(),
                                    data,
                                )
                                .expect("failed to send task to reasoner");
                        }
                    }

                    sender_to_frontend = tx;
                    current_agent = agent;
                }
            }

            // Listen for events from reasoner to send the response to frontend
            let the_events = match current_agent {
                MaeAgent::Questioner => &mut events,
                MaeAgent::WebSearch => &mut events,
                MaeAgent::PapersResearch => &mut paper_events,
            };

            '_while: while let Some(event) = the_events.recv() {
                match event {
                    Event::Input {
                        id,
                        metadata: _,
                        data,
                    } => {
                        match data.data_type() {
                            datatypes::DataType::Utf8 => {
                                // We are expecting more than one value in the response because of the options
                                // that are carried in the array in all the workflow.
                                // Here we simply discard the options and take the first value
                                let array: &StringArray =
                                    data.as_string_opt().expect("not a string array");
                                let received_string: &str = array.value(0);

                                match id.as_str() {
                                    // "paper_result" is the output identifier for the paper agent
                                    // "reasoner_result" is the output identifier for the reasoner agent
                                    "paper_result" | "reasoner_result" => {
                                        let parsed = current_agent
                                            .parse_response(received_string.to_string());
                                        sender_to_frontend
                                            .send(parsed)
                                            .expect("failed to send command");

                                        // Stop listening for events after receiving the actual response
                                        break '_while;
                                    }
                                    completed_step => {
                                        sender_to_frontend
                                            .send(MaeAgentResponse::PapersResearchUpdate(
                                                completed_step.to_string(),
                                            ))
                                            .expect("failed to send command");
                                    }
                                }
                            }
                            _other => {
                                println!("Received id: {}, data: {:#?}", id, data);
                            }
                        }
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
                        MaeAgent::PapersResearch => {
                            let response = MaeResponsePapersResearch {
                                task: task.clone(),
                                suggestion: "This is a fake response".to_string(),
                            };
                            tx.send(MaeAgentResponse::PapersResearchResponse(response))
                                .expect("failed to send command");
                        }
                    },
                }
            }
        });

        backend
    }
}
