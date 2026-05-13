use clap::Parser;
use rig_core::client::{CompletionClient, ProviderClient};
use rig_core::completion::Prompt;
use rig_core::completion::request::ToolDefinition;
use rig_core::providers::openai;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio;

#[derive(Deserialize)]
struct AddArgs {
    x: i32,
    y: i32,
}

#[derive(Deserialize, Serialize)]
struct Adder;

#[derive(Debug, thiserror::Error)]
#[error("math error")]
struct MathError;

impl Tool for Adder {
    const NAME: &'static str = "add";
    type Error = MathError;
    type Args = AddArgs;
    type Output = i32;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "add".to_string(),
            description: "两个数相加".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "第一个数" },
                    "y": { "type": "number", "description": "第二个数" }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("call add");
        Ok(args.x + args.y)
    }
}

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

    // Create OpenAI client
    let client = openai::CompletionsClient::from_env()?;

    // Create agent with a single context prompt
    let comedian_agent = client
        .agent("LongCat-Flash-Thinking-2601")
        .preamble(&args.system)
        .tool(Adder)
        .build();

    // Prompt the agent and print the response
    let response = comedian_agent.prompt(file.as_str()).await?;

    println!("{response}");

    Ok(())
}
