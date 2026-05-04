use agent_core::complexity_classifier;
use agent_core::context_manager::ContextManager;
use agent_core::http_client::HttpClient;
use agent_core::model_router::ModelRouter;
use agent_core::provider::openrouter::OpenRouterProvider;
#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Set OPENROUTER_API_KEY environment variable");
    let provider = OpenRouterProvider::new(api_key);
    let http = HttpClient::new();
    let router = ModelRouter::new(ModelRouter::default_tiers());
    let mut ctx = ContextManager::new(4000);
    ctx.set_system_prompt("You are a helpful assistant. Keep responses brief.");
    let prompt = "What is 2 + 2?";
    let complexity = complexity_classifier::classify(prompt);
    println!("Complexity: {:?}", complexity);
    println!("Sending prompt: {}", prompt);
    match router.call_with_fallback(&http, &agent_core::provider::ProviderBackend::OpenRouter(provider), prompt, "You are a helpful assistant.").await {
        Ok(response) => {
            println!("Model: {}", response.model);
            println!("Response: {}", response.content);
            println!("Tokens: {} prompt + {} completion = {} total",
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
                response.usage.total_tokens,
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
