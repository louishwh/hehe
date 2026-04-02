use std::sync::Once;

mod agent;
mod id;
mod message;
mod stream;
mod tool;

static INIT: Once = Once::new();

#[allow(dead_code)]
pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
            )
            .init();
    });
}
