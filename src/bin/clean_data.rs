use anyhow;
use llmagent::modules::money_iterator::NumberIterator;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let content = std::fs::read_to_string("tests/prompt2.txt".to_string()).unwrap();
    let iteror = NumberIterator::new(&content);
    for item in iteror {
        print!("{}", item.content);
    }
    println!("");
    Ok(())
}
