//────────────────────────────────────────────────────────────────────────────
// examples/ai_agent_builder.rs – Fluent AI Agent Builder
//────────────────────────────────────────────────────────────────────────────

use cyrup_sugars::{AsyncTask, ByteSizeExt, OneOrMany, ZeroOneOrMany, FutureExt};
use std::marker::PhantomData;
use std::time::Duration;

// Typestate markers for fluent builder
struct NoProvider;
struct HasProvider;
struct NoConfig;
struct HasConfig;

#[derive(Debug, Clone)]
pub enum Provider {
    OpenAI(String),
    Anthropic(String), 
    Local(String),
}

impl Provider {
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::OpenAI(api_key.into())
    }
    
    pub fn anthropic(api_key: impl Into<String>) -> Self {
        Self::Anthropic(api_key.into())
    }
    
    pub fn local(endpoint: impl Into<String>) -> Self {
        Self::Local(endpoint.into())
    }
}

#[derive(Debug, Clone)]
pub enum Model {
    OpenAI { variant: OpenAIModel },
    Anthropic { variant: AnthropicModel },
}

#[derive(Debug, Clone)]
pub enum OpenAIModel {
    O4Mini,
    O4,
    O4Turbo,
}

#[derive(Debug, Clone)]
pub enum AnthropicModel {
    Claude4SonnetThinking,
    Claude4Haiku,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct WebSearch {
    pub max_results: u32,
}

#[derive(Debug, Clone)]
pub struct Cargo {
    pub workspace_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Tool {
    WebSearch(WebSearch),
    Cargo(Cargo),
}

impl WebSearch {
    pub fn new() -> Self {
        Self { max_results: 10 }
    }

    pub fn with_max_results(mut self, max: u32) -> Self {
        self.max_results = max;
        self
    }
}

impl Cargo {
    pub fn new() -> Self {
        Self { workspace_path: None }
    }

    pub fn with_workspace(mut self, path: String) -> Self {
        self.workspace_path = Some(path);
        self
    }
}

impl Into<Tool> for WebSearch {
    fn into(self) -> Tool {
        Tool::WebSearch(self)
    }
}

impl Into<Tool> for Cargo {
    fn into(self) -> Tool {
        Tool::Cargo(self)
    }
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_tokens: u32,
    pub temperature: f32,
    pub cache_size: usize,
}

#[derive(Debug)]
pub struct Agent {
    pub provider: Provider,
    pub config: AgentConfig,
    pub tools: ZeroOneOrMany<Tool>,
    pub allowed_models: OneOrMany<Model>,
    pub conversation_history: std::collections::HashMap<MessageRole, String>,
}

// Fluent builder using typestate pattern
pub struct AgentBuilder<P, C> {
    provider: Option<Provider>,
    config: Option<AgentConfig>,
    tools: ZeroOneOrMany<Tool>,
    allowed_models: OneOrMany<Model>,
    conversation_history: std::collections::HashMap<MessageRole, String>,
    _phantom: PhantomData<(P, C)>,
}

impl AgentBuilder<NoProvider, NoConfig> {
    pub fn new() -> Self {
        Self {
            provider: None,
            config: None,
            tools: ZeroOneOrMany::none(),
            allowed_models: OneOrMany::one(Model::OpenAI { variant: OpenAIModel::O4 }),
            conversation_history: std::collections::HashMap::new(),
            _phantom: PhantomData,
        }
    }
}

impl<P, C> AgentBuilder<P, C> {
    pub fn with_provider(self, provider: Provider) -> AgentBuilder<HasProvider, C> {
        AgentBuilder {
            provider: Some(provider),
            config: self.config,
            tools: self.tools,
            allowed_models: self.allowed_models,
            conversation_history: self.conversation_history,
            _phantom: PhantomData,
        }
    }

    pub fn with_config(self, config: AgentConfig) -> AgentBuilder<P, HasConfig> {
        AgentBuilder {
            provider: self.provider,
            config: Some(config),
            tools: self.tools,
            allowed_models: self.allowed_models,
            conversation_history: self.conversation_history,
            _phantom: PhantomData,
        }
    }

    pub fn with_tools(mut self, tool1: impl Into<Tool>, tool2: impl Into<Tool>) -> Self {
        self.tools = ZeroOneOrMany::many(vec![tool1.into(), tool2.into()]);
        self
    }

    pub fn with_models(mut self, models: OneOrMany<Model>) -> Self {
        self.allowed_models = models;
        self
    }

    pub fn conversation_history(mut self, role1: MessageRole, msg1: impl Into<String>, role2: MessageRole, msg2: impl Into<String>) -> Self {
        self.conversation_history.insert(role1, msg1.into());
        self.conversation_history.insert(role2, msg2.into());
        self
    }
}

impl AgentBuilder<HasProvider, HasConfig> {
    pub fn build(self) -> Agent {
        Agent {
            provider: self.provider.unwrap_or_else(|| unreachable!("typestate ensures provider exists")),
            config: self.config.unwrap_or_else(|| unreachable!("typestate ensures config exists")),
            tools: self.tools,
            allowed_models: self.allowed_models,
            conversation_history: self.conversation_history,
        }
    }

    pub fn build_async(self) -> AsyncTask<Agent> {
        let agent = self.build();
        AsyncTask::from_future(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            agent
        })
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            temperature: 0.7,
            cache_size: 512.mb().as_bytes(),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Building AI Agent with cyrup_sugars...");

    // Demonstrate fluent builder with typestate
    let agent = AgentBuilder::new()
        .with_config(AgentConfig {
            max_tokens: 8192,
            temperature: 0.8,
            cache_size: 1024.mb().as_bytes(),
        })
        .with_provider(Provider::openai("sk-..."))
        .with_tools(WebSearch::new(), Cargo::new())
        .with_models(OneOrMany::many(vec![
            Model::OpenAI { variant: OpenAIModel::O4Mini },
            Model::Anthropic { variant: AnthropicModel::Claude4SonnetThinking },
        ]).unwrap_or_else(|_| OneOrMany::one(Model::OpenAI { variant: OpenAIModel::O4 })))
        .conversation_history(MessageRole::User, "hi", MessageRole::Assistant, "Hey, Dave")
        .build_async()
        .tap_ok(|agent| println!("Agent built: {:?}", agent))
        .on_result(|result| match result {
            Ok(agent) => agent,
            Err(_) => panic!("Failed to build agent")
        })
        .await;

    println!("✅ AI Agent builder example completed");
}