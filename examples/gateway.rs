use agent_core::gateway_server::GatewayServer;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("GATEWAY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8765);

    let auth = std::env::var("GATEWAY_AUTH_TOKEN").ok();

    let mut server = GatewayServer::new(port);
    if let Some(token) = auth {
        server = server.with_auth(token);
    }

    println!("Starting gateway on ws://127.0.0.1:{}", port);
    println!("Connect with: websocat ws://127.0.0.1:{}/", port);
    println!("Send: {{\"type\":\"task\",\"prompt\":\"search for Rust\"}}");

    if let Err(e) = server.start().await {
        eprintln!("Gateway error: {}", e);
    }
}
