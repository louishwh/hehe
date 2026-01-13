use axum::{
    routing::{get, post},
    Router,
};
use hehe_agent::Agent;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::ServerConfig;
use crate::error::Result;
use crate::routes;
use crate::state::AppState;

pub struct Server {
    config: ServerConfig,
    state: AppState,
}

impl Server {
    pub fn new(config: ServerConfig, agent: Agent) -> Self {
        Self {
            config,
            state: AppState::new(agent),
        }
    }

    pub fn router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .route("/health", get(routes::health))
            .route("/ready", get(routes::ready))
            .route("/api/v1/chat", post(routes::chat))
            .route("/api/v1/chat/stream", post(routes::chat_stream))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(self.state.clone())
    }

    pub async fn run(self) -> Result<()> {
        let addr = self.config.socket_addr();
        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            crate::error::ServerError::internal(format!("Failed to bind to {}: {}", addr, e))
        })?;

        info!("Server listening on {}", addr);

        axum::serve(listener, self.router())
            .await
            .map_err(|e| crate::error::ServerError::internal(e.to_string()))?;

        Ok(())
    }

    pub async fn run_with_shutdown<F>(self, shutdown: F) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let addr = self.config.socket_addr();
        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            crate::error::ServerError::internal(format!("Failed to bind to {}: {}", addr, e))
        })?;

        info!("Server listening on {}", addr);

        axum::serve(listener, self.router())
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|e| crate::error::ServerError::internal(e.to_string()))?;

        Ok(())
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received");
}
