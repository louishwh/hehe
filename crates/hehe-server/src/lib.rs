pub mod config;
pub mod error;
pub mod routes;
pub mod server;
pub mod state;

pub use config::ServerConfig;
pub use error::{Result, ServerError};
pub use server::{shutdown_signal, Server};
pub use state::AppState;

pub mod prelude {
    pub use crate::config::ServerConfig;
    pub use crate::error::{Result, ServerError};
    pub use crate::server::{shutdown_signal, Server};
    pub use crate::state::AppState;
}
