use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use super::create_agent;

pub async fn run(api_key: Option<String>, model: &str, system_prompt: &str) -> anyhow::Result<()> {
    let agent = create_agent(api_key, model, system_prompt)?;
    let session = agent.create_session();

    println!("{}", "hehe AI Agent".green().bold());
    println!("Type {} to exit\n", "quit".yellow());

    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(&format!("{} ", "You:".cyan().bold()));

        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "quit" || line == "exit" {
                    println!("{}", "Goodbye!".green());
                    break;
                }

                rl.add_history_entry(line)?;

                print!("{} ", "Assistant:".magenta().bold());

                match agent.chat(&session, line).await {
                    Ok(response) => {
                        println!("{}\n", response);
                    }
                    Err(e) => {
                        println!("{} {}\n", "Error:".red().bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".yellow());
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".green());
                break;
            }
            Err(err) => {
                println!("{} {:?}", "Error:".red().bold(), err);
                break;
            }
        }
    }

    Ok(())
}
