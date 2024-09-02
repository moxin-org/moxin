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
pub struct MaeResponseReasoner {
    pub task: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseResearchScholar {
    pub task: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseSearchAssistantResource {
    pub name: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseSearchAssistantResult {
    pub web_search_results: String,
    #[serde(deserialize_with = "parse_web_search_resource")]
    pub web_search_resource: Vec<MaeResponseSearchAssistantResource>,
}

fn parse_web_search_resource<'de, D>(
    deserializer: D,
) -> Result<Vec<MaeResponseSearchAssistantResource>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let resources: Vec<MaeResponseSearchAssistantResource> =
        serde_json::from_str(&s).map_err(serde::de::Error::custom)?;

    Ok(resources)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaeResponseSearchAssistant {
    pub task: String,
    pub result: MaeResponseSearchAssistantResult,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaeAgent {
    Reasoner,
    SearchAssistant,
    ResearchScholar,
}

pub enum MaeAgentWorkflow {
    BasicReasoner(String),
    ResearchScholar,
}

impl MaeAgent {
    pub fn name(&self) -> String {
        match self {
            MaeAgent::Reasoner => "Reasoner Agent".to_string(),
            MaeAgent::SearchAssistant => "Search Assistant".to_string(),
            MaeAgent::ResearchScholar => "Research Scholar".to_string(),
        }
    }

    pub fn workflow(&self) -> MaeAgentWorkflow {
        match self {
            MaeAgent::Reasoner => MaeAgentWorkflow::BasicReasoner("reasoner_agent.yml".to_string()),
            MaeAgent::SearchAssistant => {
                MaeAgentWorkflow::BasicReasoner("web_search_by_dspy.yml".to_string())
            }
            MaeAgent::ResearchScholar => MaeAgentWorkflow::ResearchScholar,
        }
    }

    pub fn parse_response(&self, response: String) -> MaeAgentResponse {
        match self {
            MaeAgent::Reasoner => {
                let response = serde_json::from_str::<MaeResponseReasoner>(&response).unwrap();
                MaeAgentResponse::ReasonerResponse(response)
            }
            MaeAgent::SearchAssistant => {
                let response =
                    serde_json::from_str::<MaeResponseSearchAssistant>(&response).unwrap();
                MaeAgentResponse::SearchAssistantResponse(response)
            }
            MaeAgent::ResearchScholar => {
                let response =
                    serde_json::from_str::<MaeResponseResearchScholar>(&response).unwrap();
                MaeAgentResponse::ResearchScholarResponse(response)
            }
        }
    }
}

#[derive(Debug)]
pub enum MaeAgentResponse {
    ReasonerResponse(MaeResponseReasoner),
    SearchAssistantResponse(MaeResponseSearchAssistant),
    ResearchScholarResponse(MaeResponseResearchScholar),

    // This is not a final response, it is an indication that the agent is still working
    // but some step was completed
    ResearchScholarUpdate(String),
}

pub enum MaeAgentCommand {
    SendTask(String, MaeAgent, mpsc::Sender<MaeAgentResponse>),
    CancelTask,
}

pub struct MaeBackend {
    pub command_sender: mpsc::Sender<MaeAgentCommand>,
}

impl MaeBackend {
    pub fn available_agents() -> Vec<MaeAgent> {
        vec![
            MaeAgent::Reasoner,
            MaeAgent::SearchAssistant,
            MaeAgent::ResearchScholar,
        ]
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
                        MaeAgentWorkflow::ResearchScholar => {
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
                MaeAgentCommand::CancelTask => {
                    // Cancel this loop iteration and wait for the next command
                    continue;
                }
            }

            // Listen for events from reasoner to send the response to frontend
            let the_events = match current_agent {
                MaeAgent::Reasoner | MaeAgent::SearchAssistant => &mut events,
                MaeAgent::ResearchScholar => &mut paper_events,
            };

            '_while: while let Some(event) = the_events.recv() {
                if Self::was_cancelled(&command_receiver) {
                    // Cancel this loop iteration and wait for the next command
                    // TODO send an input to the workflow to actually cancel the task in MAE
                    break '_while;
                }

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
                                            .send(MaeAgentResponse::ResearchScholarUpdate(
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

    fn was_cancelled(receiver: &mpsc::Receiver<MaeAgentCommand>) -> bool {
        match receiver.try_recv() {
            Ok(MaeAgentCommand::CancelTask) => true,
            _ => false,
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
                        MaeAgent::Reasoner => {
                            let response = MaeResponseReasoner {
                                task: task.clone(),
                                result: "This is a fake response".to_string(),
                            };
                            tx.send(MaeAgentResponse::ReasonerResponse(response))
                                .expect("failed to send command");
                        }
                        MaeAgent::SearchAssistant => {
                            let response = MaeResponseSearchAssistant {
                                task: task.clone(),
                                result: MaeResponseSearchAssistantResult {
                                    web_search_results: "This is a fake response".to_string(),
                                    web_search_resource: vec![MaeResponseSearchAssistantResource {
                                        name: "Fake resource".to_string(),
                                        url: "https://fake.com".to_string(),
                                        snippet: "This is a fake snippet".to_string(),
                                    }],
                                },
                            };
                            tx.send(MaeAgentResponse::SearchAssistantResponse(response))
                                .expect("failed to send command");
                        }
                        MaeAgent::ResearchScholar => {
                            let response = MaeResponseResearchScholar {
                                task: task.clone(),
                                suggestion: "This is a fake response".to_string(),
                            };
                            tx.send(MaeAgentResponse::ResearchScholarResponse(response))
                                .expect("failed to send command");
                        }
                    },
                    _ => (),
                }
            }
        });

        backend
    }
}
