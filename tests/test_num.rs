use llmagent::modules::money_iterator::NumberIterator;

#[test]
fn test_num() {
    let content = std::fs::read_to_string("tests/prompt.txt".to_string()).unwrap();
    let result = NumberIterator::new(&content).collect::<Vec<_>>();
    for item in &result {
        println!("{:}", item.value);
    }
    assert_eq!(result.len(), 32);
}
