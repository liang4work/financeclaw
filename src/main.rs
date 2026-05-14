use clap::Parser;
mod modules;
use modules::*;
use rig_core::client::{CompletionClient, ProviderClient};
use rig_core::completion::Prompt;
use rig_core::providers::openai;
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
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 开始对话流程
    let file = std::fs::read_to_string(&args.file)?;
    let skills_prompt = skills::get_skills()?;
    // Create OpenAI client
    let client = openai::CompletionsClient::from_env()?;

    // Create agent with a single context prompt
    let comedian_agent = client
        .agent("LongCat-Flash-Thinking-2601")
        .preamble(&(args.system + &skills_prompt))
        .tool(inter_tools::Adder)
        .tool(inter_tools::Compare)
        .build();

    // Prompt the agent and print the response
    let response = comedian_agent.prompt(file.as_str()).await?;

    println!("{response}");

    Ok(())
}
