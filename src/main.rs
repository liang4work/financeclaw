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
                    "x": { "type": "number", "description": "First number" },
                    "y": { "type": "number", "description": "Second number" }
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
    println!("start...");
    let openai_client = openai::CompletionsClient::from_env()?;
    let agent = openai_client
        .agent(openai::GPT_4O)
        .preamble("计算两个数值相加,请使用数值相加MCP")
        // .tool_server_handle(tool_server_handle)
        .tool(Adder)
        .build();

    let res = agent
        .prompt("1223与23222相加等于多少")
        .max_turns(20)
        .await?;

    println!("GPT-4o: {res}");

    Ok(())
}
