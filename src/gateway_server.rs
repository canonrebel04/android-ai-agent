use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;

/// WebSocket message protocol — client to agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "task")]
    Task {
        prompt: String,
        #[serde(default)]
        model: Option<String>,
    },
    #[serde(rename = "confirm")]
    Confirm { approved: bool },
    #[serde(rename = "status")]
    Status,
}

/// WebSocket message protocol — agent to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentMessage {
    #[serde(rename = "log")]
    Log { text: String },
    #[serde(rename = "action")]
    Action {
        skill: String,
        parameters: serde_json::Value,
    },
    #[serde(rename = "done")]
    Done { summary: String },
    #[serde(rename = "confirm_required")]
    ConfirmRequired {
        skill: String,
        reason: String,
    },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "stream")]
    Stream { text: String, model: String },
    #[serde(rename = "status")]
    Status {
        state: String,
        model: Option<String>,
    },
}

pub struct GatewayServer {
    addr: SocketAddr,
    _auth_token: Option<String>,
}

impl GatewayServer {
    pub fn new(port: u16) -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], port)),
            _auth_token: None,
        }
    }

    pub fn with_auth(mut self, token: String) -> Self {
        self._auth_token = Some(token);
        self
    }

    pub fn bind_addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }

    /// Start the WebSocket server. For each connection, spawns a handler.
    /// In production, the handler would be wired to the agent loop.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("Gateway WebSocket server listening on ws://{}", self.addr);

        let auth_token = self._auth_token.clone();

        while let Ok((stream, peer)) = listener.accept().await {
            let token = auth_token.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, token).await {
                    eprintln!("Connection error from {}: {}", peer, e);
                }
            });
        }

        Ok(())
    }
}

async fn handle_connection(
    stream: TcpStream,
    auth_token: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let callback = |req: &Request, response: Response| -> Result<Response, ErrorResponse> {
        if let Some(expected_token) = &auth_token {
            let auth_header = req.headers().get("Authorization");
            let valid = match auth_header {
                Some(val) => {
                    let token_str = val.to_str().unwrap_or("");
                    token_str == format!("Bearer {}", expected_token) || token_str == expected_token.as_str()
                }
                None => false,
            };
            if !valid {
                let mut err = Response::builder().status(401).body(Some("Unauthorized".to_string().into())).unwrap();
                err.headers_mut().insert("Content-Type", "text/plain".parse().unwrap());
                return Err(err);
            }
        }
        Ok(response)
    };

    let ws_stream = accept_hdr_async(stream, callback).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send initial status
    let status = AgentMessage::Status {
        state: "idle".to_string(),
        model: Some("claude-sonnet-4-6".to_string()),
    };
    let status_json = serde_json::to_string(&status)?;
    write.send(Message::Text(status_json.into())).await?;

    // Read loop
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if msg.is_text() {
            let text = msg.to_text()?;

            match serde_json::from_str::<ClientMessage>(text) {
                Ok(ClientMessage::Task { prompt, model }) => {
                    // In production: dispatch to agent_loop::AgentLoop::run()
                    write
                        .send(Message::Text(
                            serde_json::to_string(&AgentMessage::Log {
                                text: format!(
                                    "Received task: {} (model: {:?})",
                                    prompt,
                                    model.unwrap_or_default()
                                ),
                            })?
                            .into(),
                        ))
                        .await?;

                    // Simulate work
                    write
                        .send(Message::Text(
                            serde_json::to_string(&AgentMessage::Action {
                                skill: "web_search".into(),
                                parameters: serde_json::json!({"query": prompt}),
                            })?
                            .into(),
                        ))
                        .await?;

                    write
                        .send(Message::Text(
                            serde_json::to_string(&AgentMessage::Done {
                                summary: format!("Task completed: {}", prompt),
                            })?
                            .into(),
                        ))
                        .await?;
                }
                Ok(ClientMessage::Confirm { approved }) => {
                    write
                        .send(Message::Text(
                            serde_json::to_string(&AgentMessage::Log {
                                text: format!("Confirmation: {}", if approved { "approved" } else { "rejected" }),
                            })?
                            .into(),
                        ))
                        .await?;
                }
                Ok(ClientMessage::Status) => {
                    write
                        .send(Message::Text(
                            serde_json::to_string(&AgentMessage::Status {
                                state: "idle".into(),
                                model: Some("claude-sonnet-4-6".into()),
                            })?
                            .into(),
                        ))
                        .await?;
                }
                Err(e) => {
                    write
                        .send(Message::Text(
                            serde_json::to_string(&AgentMessage::Error {
                                message: format!("Invalid message: {}", e),
                            })?
                            .into(),
                        ))
                        .await?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_client_task() {
        let msg = ClientMessage::Task {
            prompt: "search for Rust".into(),
            model: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"task\""));
        assert!(json.contains("search for Rust"));
    }

    #[test]
    fn test_serialize_agent_done() {
        let msg = AgentMessage::Done {
            summary: "Task complete".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"done\""));
    }

    #[test]
    fn test_deserialize_client_confirm() {
        let json = r#"{"type":"confirm","approved":true}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Confirm { approved } => assert!(approved),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_server_new() {
        let server = GatewayServer::new(8765).with_auth("secret".into());
        assert!(server._auth_token.is_some());
    }
}
