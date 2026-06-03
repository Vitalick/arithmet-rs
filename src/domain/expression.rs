use std::fmt::{Display, Formatter};
use crate::domain::operation::Operation;

pub trait Expression {
    fn evaluate(&self) -> Result<String, String>;
}

impl Display for dyn Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.evaluate().unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Compare {
    pub left: i32,
    pub right: i32,
}

impl Compare {
    pub fn new(left: i32, right: i32) -> Self {
        Compare { left, right }
    }
}

impl Display for Compare {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ? {}", self.left, self.right)
    }
}

impl Expression for Compare {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DivisionWithRemainderCheck {
    pub left: i32,
    pub right: i32,
    pub quotient: i32,
}

impl DivisionWithRemainderCheck {
    pub fn new(left: i32, right: i32, quotient: i32) -> Self {
        DivisionWithRemainderCheck {
            left,
            right,
            quotient,
        }
    }
}

impl Display for DivisionWithRemainderCheck {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} / {}", self.left, self.right)
    }
}

impl Expression for DivisionWithRemainderCheck {
    fn evaluate(&self) -> Result<String, String> {
        Operation::DivisionWithRemainder.validates_operands(self.left, self.right)?;

        let remainder = self.left - self.right * self.quotient;
        if remainder == 0 {
            return Ok(format!("{} = {}", self, self.quotient));
        }

        Ok(format!(
            "{} = {} (остаток {})",
            self, self.quotient, remainder
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_division_with_remainder() {
        let division = DivisionWithRemainderCheck::new(10, 3, 3);
        assert_eq!(division.evaluate().unwrap(), "10 / 3 = 3 (остаток 1)");
    }

    #[test]
    fn test_compare_expression() {
        let cases = [
            (Compare::new(1, 3), "1 < 3"),
            (Compare::new(3, 1), "3 > 1"),
            (Compare::new(3, 3), "3 = 3"),
        ];

        for (compare, expression) in cases {
            assert_eq!(compare.evaluate().unwrap(), expression);
        }
    }

    #[test]
    fn test_division_with_remainder_check_expression() {
        let cases = [
            (
                DivisionWithRemainderCheck::new(10, 3, 3),
                "10 / 3 = 3 (остаток 1)",
            ),
            (
                DivisionWithRemainderCheck::new(10, 3, 2),
                "10 / 3 = 2 (остаток 4)",
            ),
            (DivisionWithRemainderCheck::new(12, 3, 4), "12 / 3 = 4"),
        ];

        for (check, expression) in cases {
            assert_eq!(check.evaluate().unwrap(), expression);
        }
    }

}