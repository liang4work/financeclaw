use anyhow;
use llmagent::modules::money_iterator::MoneyIterator;

#[test]
fn test_num() -> anyhow::Result<()> {
    let content = std::fs::read_to_string("tests/prompt.txt".to_string())?;
    let result = MoneyIterator::new(&content).collect::<Vec<_>>();
    for item in &result {
        println!("{:?}", item);
    }
    assert_eq!(result.len(), 32);
    Ok(())
}
