use colored::Colorize;
use hehe_server::{shutdown_signal, Server, ServerConfig};

use super::create_agent;

pub async fn run(
    api_key: Option<String>,
    model: &str,
    system_prompt: &str,
    host: &str,
    port: u16,
) -> anyhow::Result<()> {
    let agent = create_agent(api_key, model, system_prompt)?;

    let config = ServerConfig::new()
        .with_host(host)
        .with_port(port);

    println!("{}", "Starting hehe server...".green().bold());
    println!("Listening on {}:{}", host.cyan(), port.to_string().cyan());
    println!("Press {} to stop\n", "Ctrl+C".yellow());

    let server = Server::new(config, agent);
    server
        .run_with_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    println!("\n{}", "Server stopped.".green());
    Ok(())
}
