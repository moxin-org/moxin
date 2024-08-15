
pub struct Agent {
    name: String,
    description: String,
}

impl Agent {
    pub fn available_agents() -> Vec<Agent> {
        vec![
            Agent {
                name: "Questioner".to_string(),
                description: "A question answering agent".to_string(),
            },
            Agent {
                name: "WebSearch".to_string(),
                description: "A web search agent".to_string(),
            },
        ]
    }
}