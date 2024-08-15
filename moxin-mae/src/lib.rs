use dora_node_api::{
    self,
    arrow::array::StringArray,
    dora_core::config::{DataId, NodeId},
    DoraNode, Event, MetadataParameters,
};
use eyre::Context;

use std::sync::mpsc::{self, channel};

pub struct MoxinMae {
    // command_receiver: mpsc::Receiver<String>,
    pub command_sender: mpsc::Sender<(String, mpsc::Sender<String>)>,
}

impl MoxinMae {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = channel();

        let (tx, rx) = channel();

        let mae = Self { command_sender };

        std::thread::spawn(move || {
            Self::start_receiver_loop(tx);
        });

        std::thread::spawn(move || {
            Self::start_sender_loop(command_receiver, rx);
        });

        mae
    }

    pub fn start_receiver_loop(sender: mpsc::Sender<String>) {
        loop {
            if let Ok((node, mut events)) =
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
        command_receiver: mpsc::Receiver<(String, mpsc::Sender<String>)>,
        rx: mpsc::Receiver<String>,
    ) {
        if let Ok((mut node, _events)) =
            DoraNode::init_from_node_id(NodeId::from("reasoner_task_input".to_string()))
        {
            loop {
                let (buffer, tx) = command_receiver.recv().unwrap();

                let data = StringArray::from(vec![buffer.trim().to_string()]);

                let res = node.send_output(
                    DataId::from("reasoner_task".to_string()),
                    MetadataParameters::default(),
                    data,
                );
                dbg!(res.unwrap());

                match rx.recv() {
                    Ok(response) => {
                        tx.send(response).unwrap();
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                    }
                }
            }
        }
    }
}
