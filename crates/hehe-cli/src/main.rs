use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "hehe")]
#[command(author, version, about = "hehe AI Agent CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, env = "OPENAI_API_KEY")]
    api_key: Option<String>,

    #[arg(short, long, global = true, default_value = "gpt-4o")]
    model: String,

    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive chat session
    Chat {
        /// System prompt for the agent
        #[arg(short, long, default_value = "You are a helpful assistant.")]
        system: String,
    },
    /// Run a single message and exit
    Run {
        /// The message to send
        message: String,
        /// System prompt for the agent
        #[arg(short, long, default_value = "You are a helpful assistant.")]
        system: String,
    },
    /// Start the HTTP server
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// System prompt for the agent
        #[arg(short, long, default_value = "You are a helpful assistant.")]
        system: String,
    },
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level)),
        )
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let api_key = cli.api_key.or_else(|| std::env::var("OPENAI_API_KEY").ok());

    match cli.command {
        Commands::Chat { system } => {
            commands::chat::run(api_key, &cli.model, &system).await?;
        }
        Commands::Run { message, system } => {
            commands::run::run(api_key, &cli.model, &system, &message).await?;
        }
        Commands::Serve { host, port, system } => {
            commands::serve::run(api_key, &cli.model, &system, &host, port).await?;
        }
    }

    Ok(())
}
