use anyhow::Result;
use clap::Parser;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::openai;
use tokio;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// system prompt
    #[arg(short, long)]
    system: String,

    /// prompt file
    #[arg(short, long)]
    file: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let file = std::fs::read_to_string(&args.file)?;

    // Create OpenAI client
    let client = openai::CompletionsClient::from_env()?;

    // Create agent with a single context prompt
    let comedian_agent = client
        .agent("LongCat-Flash-Thinking-2601")
        .preamble(&args.system)
        .build();

    // Prompt the agent and print the response
    let response = comedian_agent.prompt(file.as_str()).await?;

    println!("{response}");

    Ok(())
}
