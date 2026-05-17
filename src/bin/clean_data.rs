use anyhow;
use clap::Parser;
use llmagent::modules::money_iterator::NumberIterator;
use std::io::Write;
use tokio;

#[derive(Parser)]
#[command(name = "clean_data")]
struct Cli {
    /// 输入文件路径
    #[arg(short, long)]
    input: String,

    /// 输出文件路径
    #[arg(short, long)]
    output: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let content = std::fs::read_to_string(&cli.input)?;
    let iteror = NumberIterator::new(&content);

    let mut out = std::fs::File::create(&cli.output)?;
    for item in iteror {
        write!(out, "{}", item.content)?;
    }
    writeln!(out)?;

    Ok(())
}
