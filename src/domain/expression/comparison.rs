use std::fmt::{Display, Formatter};
use super::expression::Expression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comparison {
    pub left: i64,
    pub right: i64,
}

impl Comparison {
    pub fn new(left: i64, right: i64) -> Self {
        Comparison { left, right }
    }
}

impl Display for Comparison {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ? {}", self.left, self.right)
    }
}

impl Expression for Comparison {
    fn evaluate(&self) -> Result<String, String> {
        let symbol;
        if self.left > self.right {
            symbol = ">"
        } else if self.left < self.right {
            symbol = "<"
        } else {
            symbol = "="
        }
        Ok(format!("{} {} {}", self.left, symbol, self.right))
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_compare_expression() {
        let cases = [
            (Comparison::new(1, 3), "1 < 3"),
            (Comparison::new(3, 1), "3 > 1"),
            (Comparison::new(3, 3), "3 = 3"),
        ];

        for (compare, expression) in cases {
            assert_eq!(compare.evaluate().unwrap(), expression);
        }
    }
}