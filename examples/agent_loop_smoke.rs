use agent_core::agent_loop::{AgentLoop, AgentLoopConfig};
use agent_core::complexity_classifier;
use agent_core::context_manager::ContextManager;
use agent_core::http_client::HttpClient;
use agent_core::identity;
use agent_core::model_router::ModelRouter;
use agent_core::provider::openrouter::OpenRouterProvider;
#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Set OPENROUTER_API_KEY");
    let provider = OpenRouterProvider::new(api_key);
    let http = HttpClient::new();
    let router = ModelRouter::new(ModelRouter::default_tiers());
    let mut ctx = ContextManager::new(4000);
    let prompt = "Search the web for the latest Rust release version";
    let complexity = complexity_classifier::classify(prompt);
    let system_prompt = identity::build_system_prompt(
        complexity,
        "You control an Android phone. Use tools to complete tasks.",
        "User prefers concise responses.",
    );
    let mut agent = AgentLoop::new(AgentLoopConfig {
        max_steps: 3,
        ..Default::default()
    });
    match agent.run(&http, &provider, &router, &mut ctx, prompt, &system_prompt).await {
        Ok(result) => {
            println!("Agent completed: {}", result);
            println!("\nEvent log:");
            for event in agent.events() {
                println!("  [{}] {:?}", event.event_type(), event);
            }
        }
        Err(e) => {
            eprintln!("Agent failed: {}", e);
        }
    }
}
