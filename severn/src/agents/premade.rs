use crate::agents::traits::Agent;

struct ArticleWriter {
    target_audience: String,
    tone: String,
}

impl ArticleWriter {
    pub fn new() -> Self {
        Self {
            target_audience: "software developers".to_string(),
            tone: "concise".to_string(),
        }
    }

    pub fn with_target_audience(mut self, target_audience: &str) -> Self {
        self.target_audience = target_audience.to_owned();

        self
    }

    pub fn target_audience(&self) -> &str {
        &self.target_audience
    }

    pub fn with_tone(mut self, tone: &str) -> Self {
        self.tone = tone.to_owned();

        self
    }
    pub fn tone(&self) -> &str {
        &self.tone
    }
}

impl Agent for ArticleWriter {
    fn name(&self) -> String {
        "ArticleWriter".into()
    }

    fn system_message(&self) -> String {
        format!("You are an AI agent.

         Your job is to write an article that involves the data (or summary) that you've been given. Your target audience is {}.

        When answering, your tone should be: {}.",
                self.target_audience(),
                self.tone()
        )
    }
}
