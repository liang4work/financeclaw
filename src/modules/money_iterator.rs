const MUNBERS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', '零', '一', '二', '三', '四', '五',
    '六', '七', '八', '九', '十', '百', '千', '万', '亿', '壹', '贰', '叁', '肆', '伍', '陆', '柒',
    '捌', '玖', '拾',
];

/// 触发字符：检测到这些字符才认为是金额
const TRIGGERS: &[char] = &['元', '万', '亿', '%'];

pub struct NumberIterator<'a> {
    data: &'a str,
    index: usize,
}

impl<'a> NumberIterator<'a> {
    pub fn new(data: &'a str) -> Self {
        Self { data, index: 0 }
    }

    fn is_money_char(&self, c: char) -> bool {
        MUNBERS.iter().any(|&mc| mc == c)
    }

    fn is_trigger(&self, c: char) -> bool {
        TRIGGERS.iter().any(|&t| t == c)
    }

    /// 从 pos 位置向前（向左）找到上一个字符的起始字节索引
    fn prev_char_boundary(&self, pos: usize) -> Option<usize> {
        if pos == 0 {
            return None;
        }
        let bytes = self.data.as_bytes();
        let mut i = pos - 1;
        while i > 0 && bytes[i] & 0xC0 == 0x80 {
            i -= 1;
        }
        Some(i)
    }

    /// 将原始匹配文本拆分为阿拉伯数字部分和中文数字部分，再合并为统一数值
    fn split_raw(raw: &str) -> NumberItem {
        let mut alpha = String::new();
        let mut ch_char = String::new();
        for c in raw.chars() {
            if c == ',' || c == '元' {
                continue;
            }
            if c.is_ascii_digit() || c == '.' {
                alpha.push(c);
            } else {
                ch_char.push(c);
            }
        }

        // 解析阿拉伯数字部分
        let alpha_value: f64 = if alpha.is_empty() {
            1.0
        } else {
            alpha.parse().unwrap_or(1.0)
        };

        // 解析中文数字部分
        let chinese_value = chinese_to_number(&ch_char);

        NumberItem {
            value: alpha_value * chinese_value,
        }
    }
}

/// 判断字符是否为中文数字字符
fn is_chinese_number_char(c: char) -> bool {
    matches!(
        c,
        '零' | '一'
            | '二'
            | '三'
            | '四'
            | '五'
            | '六'
            | '七'
            | '八'
            | '九'
            | '十'
            | '百'
            | '千'
            | '万'
            | '亿'
            | '壹'
            | '贰'
            | '叁'
            | '肆'
            | '伍'
            | '陆'
            | '柒'
            | '捌'
            | '玖'
            | '拾'
            | '佰'
            | '仟'
    )
}

/// 中文数字字符 → 阿拉伯数字（0-9）
fn chinese_digit(c: char) -> Option<u64> {
    match c {
        '零' => Some(0),
        '一' | '壹' => Some(1),
        '二' | '贰' => Some(2),
        '三' | '叁' => Some(3),
        '四' | '肆' => Some(4),
        '五' | '伍' => Some(5),
        '六' | '陆' => Some(6),
        '七' | '柒' => Some(7),
        '八' | '捌' => Some(8),
        '九' | '玖' => Some(9),
        _ => None,
    }
}

/// 解析小于 10000 的纯中文数字（不含"万""亿"）
/// 如 "三百五十六" → 356, "一百二十三" → 123
fn parse_small_number(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut total = 0.0_f64;
    let mut current = 0.0_f64;

    for c in s.chars() {
        if c == '零' {
            continue;
        } else if let Some(d) = chinese_digit(c) {
            total += current;
            current = d as f64;
        } else if c == '十' || c == '拾' {
            if current == 0.0 {
                current = 1.0;
            }
            current *= 10.0;
        } else if c == '百' || c == '佰' {
            if current == 0.0 {
                current = 1.0;
            }
            current *= 100.0;
        } else if c == '千' || c == '仟' {
            if current == 0.0 {
                current = 1.0;
            }
            current *= 1000.0;
        }
    }
    total += current;
    total
}

/// 递归解析带 "亿" 的中文数字（亿内可能含万）
fn parse_before_yi(s: &str) -> f64 {
    if s.is_empty() {
        return 1.0; // "亿" 前无数字表示 1 亿
    }
    match s.split_once('万') {
        Some((left, right)) => {
            let left_val = if left.is_empty() {
                1.0
            } else {
                parse_small_number(left)
            };
            let right_val = parse_small_number(right);
            left_val * 10_000.0 + right_val
        }
        None => parse_small_number(s),
    }
}

/// 将完整中文数字字符串转换为数值
/// 支持 "亿" "万" 组合，如 "一百万亿零三千亿" → 100300000000000
fn parse_chinese(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    match s.split_once('亿') {
        Some((left, right)) => {
            let left_val = parse_before_yi(left);
            let right_val = parse_chinese(right);
            left_val * 100_000_000.0 + right_val
        }
        None => {
            // 无 "亿"，尝试按 "万" 拆分
            match s.split_once('万') {
                Some((left, right)) => {
                    let left_val = if left.is_empty() {
                        1.0
                    } else {
                        parse_small_number(left)
                    };
                    let right_val = parse_small_number(right);
                    left_val * 10_000.0 + right_val
                }
                None => parse_small_number(s),
            }
        }
    }
}

/// 将中文数字字符串转为 f64 统一数值
/// 若字符串不含任何中文数字字符（如 "%"），视为单位标记，返回 1.0
fn chinese_to_number(s: &str) -> f64 {
    if s.is_empty() {
        return 1.0;
    }
    if !s.chars().any(is_chinese_number_char) {
        return 1.0; // 纯非中文数字字符（如 "%"）视为单位标记
    }
    let result = parse_chinese(s);
    if result == 0.0 { 1.0 } else { result }
}

#[derive(Debug, PartialEq)]
pub struct NumberItem {
    pub value: f64,
}

impl<'a> Iterator for NumberIterator<'a> {
    type Item = NumberItem;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.data.len();

        // 1. 正向扫描找到触发字符
        while self.index < len {
            let c = self.data[self.index..].chars().next()?;
            if self.is_trigger(c) {
                break;
            }
            self.index += c.len_utf8();
        }

        if self.index >= len {
            return None;
        }

        let trigger_start = self.index;
        let trigger_char = self.data[trigger_start..].chars().next()?;
        let trigger_len = trigger_char.len_utf8();

        // 2. 从触发字符之前开始反向扫描，收集连续的数字/数值字符
        let money_start = {
            let mut start = trigger_start;
            while let Some(prev) = self.prev_char_boundary(start) {
                let c = self.data[prev..start].chars().next()?;
                if self.is_money_char(c) {
                    start = prev;
                } else {
                    break;
                }
            }
            start
        };

        // 3. 从触发字符之后继续正向扫描，收集后续的数字/数值/触发字符
        //    （如 "一百万亿" 中 "万" 后面的 "亿"，"200万元" 中 "万" 后面的 "元"）
        let mut end_pos = trigger_start + trigger_len;
        while end_pos < len {
            let c = self.data[end_pos..].chars().next()?;
            if self.is_money_char(c) || self.is_trigger(c) {
                end_pos += c.len_utf8();
            } else {
                break;
            }
        }

        self.index = end_pos;
        let raw = &self.data[money_start..end_pos];
        Some(Self::split_raw(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(value: f64) -> NumberItem {
        NumberItem { value }
    }

    #[test]
    fn test_empty_string() {
        let mut iter = NumberIterator::new("");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_no_money_text() {
        let mut iter = NumberIterator::new("今天天气真好，我们去公园散步吧。");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_simple_number() {
        // "元" 触发 → 反向找到 "123.45" → 数值 123.45
        let mut iter = NumberIterator::new("花费了123.45元");
        assert_eq!(iter.next(), Some(item(123.45)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_chinese_number() {
        // "元" 触发 → 反向找到 "一百二十三" → 数值 123
        let mut iter = NumberIterator::new("一共一百二十三元");
        assert_eq!(iter.next(), Some(item(123.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_chinese_number_clean() {
        // "元" 触发 → 反向找到 "五百" → 数值 500
        let mut iter = NumberIterator::new("花费了五百元");
        assert_eq!(iter.next(), Some(item(500.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_multiple_amounts() {
        let mut iter = NumberIterator::new("收入100元，支出200.50元，结余-50元");
        assert_eq!(iter.next(), Some(item(100.0)));
        assert_eq!(iter.next(), Some(item(200.5)));
        assert_eq!(iter.next(), Some(item(50.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_mixed_digital_and_chinese() {
        // "123" 无触发字符 → 跳过；"四百五十六" 无触发字符 → 跳过
        let mut iter = NumberIterator::new("数字123，中文四百五十六");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_mixed_with_triggers() {
        let mut iter = NumberIterator::new("价格123元，合计四百五十六万");
        assert_eq!(iter.next(), Some(item(123.0)));
        // "万" 触发 → "四百五十六万" → 456 × 10000 = 4560000
        assert_eq!(iter.next(), Some(item(4_560_000.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_amount_with_comma() {
        let mut iter = NumberIterator::new("总额1,234,567.89元");
        assert_eq!(iter.next(), Some(item(1_234_567.89)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_uppercase_chinese() {
        // "万" 触发 → "壹万贰" → 1×10000 + 2 = 10002
        // "元" 触发 → "肆拾伍" → 45
        let mut iter = NumberIterator::new("人民币壹万贰仟叁佰肆拾伍元整");
        assert_eq!(iter.next(), Some(item(10_002.0)));
        assert_eq!(iter.next(), Some(item(45.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_amount_at_start() {
        let mut iter = NumberIterator::new("99.9元起");
        assert_eq!(iter.next(), Some(item(99.9)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_amount_at_end_without_trigger() {
        // "五百" 末尾无触发字符 → 不认为是金额
        let mut iter = NumberIterator::new("总计：五百");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_only_numbers_without_trigger() {
        // 纯数字无触发字符 → 不认为是金额
        let mut iter = NumberIterator::new("1234567890");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_adjacent_amounts_with_separator() {
        // "100" 和 "200" 后面都没有触发字符 → 不认为是金额
        let mut iter = NumberIterator::new("前款100后款200");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_adjacent_amounts_with_trigger() {
        let mut iter = NumberIterator::new("前款100元后款200万元");
        assert_eq!(iter.next(), Some(item(100.0)));
        // "200万" → 200 × 10000 = 2000000
        assert_eq!(iter.next(), Some(item(2_000_000.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_collect_all() {
        let results: Vec<NumberItem> =
            NumberIterator::new("苹果10元，香蕉20.5元，合计三十元五角").collect();
        // "10元": → 10
        // "20.5元": → 20.5
        // "三十元五": → 35
        assert_eq!(results, vec![item(10.0), item(20.5), item(35.0)]);
    }

    #[test]
    fn test_multi_char_non_money_interleaved() {
        // 无触发字符 → 不认为是金额
        let mut iter = NumberIterator::new("数字3.14和九十九以及1,000");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_chinese_big_numbers() {
        // "万" 触发 → "一百万亿零三千亿" → 100300000000000
        let mut iter = NumberIterator::new("全国GDP达到一百万亿零三千亿");
        assert_eq!(iter.next(), Some(item(100_300_000_000_000.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_billion_trigger() {
        let mut iter = NumberIterator::new("去年利润五十亿元");
        assert_eq!(iter.next(), Some(item(5_000_000_000.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_percent_trigger() {
        // "25%" → "%" 是单位标记，数值为 25
        let mut iter = NumberIterator::new("增长率达到25%");
        assert_eq!(iter.next(), Some(item(25.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_mixed_alpha_and_chinese() {
        // "3000万元" → 3000 × 10000 = 30000000
        let mut iter = NumberIterator::new("项目投资3000万元");
        assert_eq!(iter.next(), Some(item(30_000_000.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_alpha_chinese_mixed() {
        // "1.5亿" → 1.5 × 100000000 = 150000000
        let mut iter = NumberIterator::new("数据1.5亿人次");
        assert_eq!(iter.next(), Some(item(150_000_000.0)));
        assert_eq!(iter.next(), None);
    }
}
