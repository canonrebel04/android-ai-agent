//! Orchestrator for the unified chat experience.
//! Combines AgentLoop, SoulSystem, and MemoryManager into a high-level API for JNI.

use crate::agent_loop::{AgentLoop, AgentLoopConfig};
use crate::complexity_classifier::{classify, suggest_model};
use crate::context_manager::ContextManager;
use crate::http_client::HttpClient;
use crate::memory_manager::MemoryManager;
use crate::model_router::ModelRouter;
use crate::provider::{openrouter::OpenRouterProvider, ProviderBackend};
use crate::soul::SoulSystem;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct UnifiedAgent {
    pub agent_loop: Mutex<AgentLoop>,
    pub context: Mutex<ContextManager>,
    pub soul: SoulSystem,
    pub memory: MemoryManager,
    pub router: ModelRouter,
    pub http: HttpClient,
    pub provider: Mutex<Option<ProviderBackend>>,
}

static GLOBAL_AGENT: Lazy<UnifiedAgent> = Lazy::new(|| {
    UnifiedAgent {
        agent_loop: Mutex::new(AgentLoop::new(AgentLoopConfig::default())),
        context: Mutex::new(ContextManager::new(20)), // 20 messages window
        soul: SoulSystem::new(),
        memory: MemoryManager::new(),
        router: ModelRouter::new(ModelRouter::default_tiers()),
        http: HttpClient::new(),
        provider: Mutex::new(None),
    }
});

pub fn get_agent() -> &'static UnifiedAgent {
    &GLOBAL_AGENT
}

impl UnifiedAgent {
    pub fn init(&self, openrouter_key: String) {
        let mut p = self.provider.lock().unwrap();
        *p = Some(ProviderBackend::OpenRouter(OpenRouterProvider::new(
            openrouter_key,
        )));
    }

    pub async fn process_message(&self, text: String) -> String {
        let provider_guard = self.provider.lock().unwrap();
        let Some(ref backend) = *provider_guard else {
            return "Error: Agent not initialized with API key".to_string();
        };

        let mut loop_guard = self.agent_loop.lock().unwrap();
        let mut ctx_guard = self.context.lock().unwrap();

        // Classify the prompt and log suggested model
        let complexity = classify(&text);
        let suggested = suggest_model(complexity);
        println!("Prompt classified as {:?}, suggested model: {}", complexity, suggested);

        // Assemble system prompt with SOUL bootstrap
        let bootstrap = self.soul.assemble_prompt(
            "You are Hermes, a high-performance Android AI agent. Help the user with technical tasks.",
            None
        );

        match loop_guard
            .run(
                &self.http,
                backend,
                &self.router,
                &mut ctx_guard,
                &text,
                &bootstrap.system_prompt,
            )
            .await
        {
            Ok(response) => {
                self.soul.mark_first_turn_done();
                response
            }
            Err(e) => format!("Error: {:?}", e),
        }
    }
}
