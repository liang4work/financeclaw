use rig_core::completion::request::ToolDefinition;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use thiserror;

#[derive(Deserialize)]
pub struct CompareArgs {
    x: i32,
    y: i32,
}

#[derive(Debug, thiserror::Error)]
#[error("math error")]
pub struct MathError;

#[derive(Deserialize, Serialize)]
pub struct Compare;

impl Tool for Compare {
    const NAME: &'static str = "compare";
    type Error = MathError;
    type Args = CompareArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "compare".to_string(),
            description: "比较两个数的大小".to_string(),
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
        println!("call compare");
        match args.x.cmp(&args.y) {
            Ordering::Greater => Ok(format!("{} 大于 {}", args.x, args.y)),
            Ordering::Equal => Ok(format!("{} 等于 {}", args.x, args.y)),
            Ordering::Less => Ok(format!("{} 小于 {}", args.x, args.y)),
        }
    }
}

#[derive(Deserialize)]
pub struct AddArgs {
    x: i32,
    y: i32,
}

#[derive(Deserialize, Serialize)]
pub struct Adder;

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
