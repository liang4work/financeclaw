use anyhow;
use llmagent::modules::money_iterator::NumberIterator;

#[test]
fn test_num() -> anyhow::Result<()> {
    let content = std::fs::read_to_string("tests/prompt.txt".to_string())?;
    let result = NumberIterator::new(&content).collect::<Vec<_>>();
    for item in &result {
        println!("{:} {:}", item.alpha, item.ch_char);
    }
    assert_eq!(result.len(), 32);
    Ok(())
}
