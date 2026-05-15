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

    /// 将原始匹配文本拆分为阿拉伯数字部分和中文数字部分
    fn split_raw(raw: &str) -> NumberItem {
        let mut alpha = String::new();
        let mut ch_char = String::new();
        for c in raw.chars() {
            if c.is_ascii_digit() || c == '.' || c == ',' {
                alpha.push(c);
            } else {
                // 忽略最后一个字符
                if c == '元' {
                    break;
                }
                ch_char.push(c);
            }
        }
        NumberItem { alpha, ch_char }
    }
}

#[derive(Debug, PartialEq)]
pub struct NumberItem {
    pub alpha: String,
    pub ch_char: String,
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

    fn item(alpha: &str, ch_char: &str) -> NumberItem {
        NumberItem {
            alpha: alpha.into(),
            ch_char: ch_char.into(),
        }
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
        // "元" 触发 → 反向找到 "123.45" → 结果 "123.45元"
        let mut iter = NumberIterator::new("花费了123.45元");
        assert_eq!(iter.next(), Some(item("123.45", "元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_chinese_number() {
        // "元" 触发 → 反向找到 "一百二十三" → 结果 "一百二十三元"
        let mut iter = NumberIterator::new("一共一百二十三元");
        assert_eq!(iter.next(), Some(item("", "一百二十三元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_chinese_number_clean() {
        // "元" 触发 → 反向找到 "五百" → 结果 "五百元"
        let mut iter = NumberIterator::new("花费了五百元");
        assert_eq!(iter.next(), Some(item("", "五百元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_multiple_amounts() {
        let mut iter = NumberIterator::new("收入100元，支出200.50元，结余-50元");
        assert_eq!(iter.next(), Some(item("100", "元")));
        assert_eq!(iter.next(), Some(item("200.50", "元")));
        assert_eq!(iter.next(), Some(item("50", "元")));
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
        assert_eq!(iter.next(), Some(item("123", "元")));
        // "万" 触发 → 反向找到 "四百五十六" → 结果 "四百五十六万"
        assert_eq!(iter.next(), Some(item("", "四百五十六万")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_amount_with_comma() {
        let mut iter = NumberIterator::new("总额1,234,567.89元");
        assert_eq!(iter.next(), Some(item("1,234,567.89", "元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_uppercase_chinese() {
        // "万" 触发 → 反向 "壹" → 正向 "贰" (遇"仟"停) → "壹万贰"
        // "元" 触发 → 反向 "肆拾伍" → 正向无 → "肆拾伍元"
        let mut iter = NumberIterator::new("人民币壹万贰仟叁佰肆拾伍元整");
        assert_eq!(iter.next(), Some(item("", "壹万贰")));
        assert_eq!(iter.next(), Some(item("", "肆拾伍元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_amount_at_start() {
        let mut iter = NumberIterator::new("99.9元起");
        assert_eq!(iter.next(), Some(item("99.9", "元")));
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
        assert_eq!(iter.next(), Some(item("100", "元")));
        assert_eq!(iter.next(), Some(item("200", "万元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_collect_all() {
        let results: Vec<NumberItem> =
            NumberIterator::new("苹果10元，香蕉20.5元，合计三十元五角").collect();
        // "10元": "元"触发, 反向"10"
        // "20.5元": "元"触发, 反向"20.5"
        // "三十元五": "元"触发, 反向"三十", 正向"五" (MUNBERS), 遇"角"停
        assert_eq!(
            results,
            vec![item("10", "元"), item("20.5", "元"), item("", "三十元五"),]
        );
    }

    #[test]
    fn test_multi_char_non_money_interleaved() {
        // 无触发字符 → 不认为是金额
        let mut iter = NumberIterator::new("数字3.14和九十九以及1,000");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_chinese_big_numbers() {
        // "万" 触发 → 反向 "一百" → 正向 "亿零三千亿" → 结果 "一百万亿零三千亿"
        let mut iter = NumberIterator::new("全国GDP达到一百万亿零三千亿");
        assert_eq!(iter.next(), Some(item("", "一百万亿零三千亿")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_billion_trigger() {
        let mut iter = NumberIterator::new("去年利润五十亿元");
        assert_eq!(iter.next(), Some(item("", "五十亿元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_percent_trigger() {
        let mut iter = NumberIterator::new("增长率达到25%");
        assert_eq!(iter.next(), Some(item("25", "%")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_mixed_alpha_and_chinese() {
        // "3000万元" = 阿拉伯 "3000" + 中文 "万元"
        let mut iter = NumberIterator::new("项目投资3000万元");
        assert_eq!(iter.next(), Some(item("3000", "万元")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_alpha_chinese_mixed() {
        // "1.5亿" → "亿"触发，反向 "1.5"，正向遇"人"（非MUNBERS）停
        let mut iter = NumberIterator::new("数据1.5亿人次");
        assert_eq!(iter.next(), Some(item("1.5", "亿")));
        assert_eq!(iter.next(), None);
    }
}
