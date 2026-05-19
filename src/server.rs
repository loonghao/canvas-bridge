use std::sync::Arc;

use anyhow::Result;
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::to_string;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info};

use crate::{
    config::Config,
    gateway::{GatewayClient, extract_job_id},
    protocol::{CanvasCommand, CanvasEvent},
};

#[derive(Clone)]
struct AppState {
    gateway: Arc<GatewayClient>,
}

pub async fn serve(config: Config) -> Result<()> {
    let state = AppState {
        gateway: Arc::new(GatewayClient::new(config.gateway)),
    };
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/ws", get(ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    info!(bind = %config.bind, "starting canvas-bridge");
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    if let Err(err) = send_event(
        &mut sender,
        CanvasEvent::Ready {
            gateway: state.gateway.base_url().to_string(),
        },
    )
    .await
    {
        debug!(%err, "failed to send ready event");
        return;
    }

    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let event = match serde_json::from_str::<CanvasCommand>(&text) {
                    Ok(command) => {
                        if let Some(started) = started_event(&command)
                            && let Err(err) = send_event(&mut sender, started).await
                        {
                            debug!(%err, "websocket send failed");
                            break;
                        }
                        handle_command(command, &state.gateway).await
                    }
                    Err(err) => CanvasEvent::Error {
                        message: format!("invalid canvas command: {err}"),
                    },
                };
                if let Err(err) = send_event(&mut sender, event).await {
                    debug!(%err, "websocket send failed");
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(err) => {
                error!(%err, "websocket receive failed");
                break;
            }
        }
    }
}

fn started_event(command: &CanvasCommand) -> Option<CanvasEvent> {
    match command {
        CanvasCommand::RunNode {
            run_id,
            node_id,
            tool_slug,
            ..
        } => Some(CanvasEvent::NodeStarted {
            run_id: run_id.clone(),
            node_id: node_id.clone(),
            tool_slug: tool_slug.clone(),
        }),
        _ => None,
    }
}

async fn handle_command(command: CanvasCommand, gateway: &GatewayClient) -> CanvasEvent {
    match command {
        CanvasCommand::SearchTools {
            query,
            dcc_type,
            limit,
        } => match gateway
            .search_tools(&query, dcc_type.as_deref(), limit)
            .await
        {
            Ok(result) => CanvasEvent::SearchResult { result },
            Err(err) => CanvasEvent::Error {
                message: err.to_string(),
            },
        },
        CanvasCommand::DescribeTool { tool_slug } => {
            match gateway.describe_tool(&tool_slug).await {
                Ok(result) => CanvasEvent::DescribeResult { tool_slug, result },
                Err(err) => CanvasEvent::Error {
                    message: err.to_string(),
                },
            }
        }
        CanvasCommand::RunNode {
            run_id,
            node_id,
            tool_slug,
            arguments,
        } => {
            let progress_token = format!("{run_id}:{node_id}");
            match gateway
                .call_tool(&tool_slug, arguments, &progress_token)
                .await
            {
                Ok(result) => CanvasEvent::NodeAccepted {
                    run_id,
                    node_id,
                    job_id: extract_job_id(&result),
                    result,
                },
                Err(err) => CanvasEvent::Error {
                    message: err.to_string(),
                },
            }
        }
    }
}

async fn send_event(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: CanvasEvent,
) -> Result<()> {
    sender
        .send(Message::Text(to_string(&event)?.into()))
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!(%err, "failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => error!(%err, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
