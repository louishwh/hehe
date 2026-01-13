use colored::Colorize;

use super::create_agent;

pub async fn run(
    api_key: Option<String>,
    model: &str,
    system_prompt: &str,
    message: &str,
) -> anyhow::Result<()> {
    let agent = create_agent(api_key, model, system_prompt)?;
    let session = agent.create_session();

    match agent.chat(&session, message).await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }

    Ok(())
}
