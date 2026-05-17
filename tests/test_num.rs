use anyhow;
use chinese_number::{ChineseCountMethod, ChineseToNumber};
use llmagent::modules::money_iterator::NumberIterator;
use std::f64;

#[test]
fn test_num() -> anyhow::Result<()> {
    let content = std::fs::read_to_string("tests/prompt.txt".to_string())?;
    let result = NumberIterator::new(&content).collect::<Vec<_>>();
    for item in &result {
        let alpha: f64 = item.alpha.parse()?;
        println!("{:}", alpha);
        if item.ch_char != "%" {
            let ch_num: f64 = item
                .ch_char
                .to_number(ChineseCountMethod::TenThousand)
                .unwrap();
            println!("{:}", alpha * ch_num);
        } else {
            println!("{:}{:}", alpha, item.ch_char);
        }
    }
    assert_eq!(result.len(), 32);
    Ok(())
}
