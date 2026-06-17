use std::fmt::{Display, Formatter};
use super::expression::Expression;
use crate::domain::operation::Operation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FakeExercise {
    pub left: i64,
    pub operation: Operation,
    pub right: i64,
    pub answer: i64,
    pub remainder: i64,
}

impl FakeExercise {
    pub fn new(left: i64, operation: Operation, right: i64, answer: i64) -> Self {
        FakeExercise {
            left,
            operation,
            right,
            answer,
            remainder: 0,
        }
    }

    pub fn new_with_remainer(
        left: i64,
        operation: Operation,
        right: i64,
        answer: i64,
        remainder: i64,
    ) -> Self {
        FakeExercise {
            left,
            operation,
            right,
            answer,
            remainder,
        }
    }

    pub fn with_remainder(self, remainder: i64) -> Self {
        FakeExercise { remainder, ..self }
    }

    pub fn with_answer(self, answer: i64) -> Self {
        FakeExercise { answer, ..self }
    }
}


impl Display for FakeExercise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.operation, self.right)
    }
}

impl Expression for FakeExercise {
    fn evaluate(&self) -> Result<String, String> {
        Operation::DivisionWithRemainder.validates_operands(self.left, self.right)?;
        match self.remainder {
            0 => Ok(format!("{} = {}", self, self.answer)),
            _ => Ok(format!(
                "{} = {} (остаток {})",
                self, self.answer, self.remainder
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_division_with_remainder_check_expression() {
        let cases = [
            (
                FakeExercise::new_with_remainer(10, Operation::Division, 3, 3, 1),
                "10 / 3 = 3 (остаток 1)",
            ),
            (
                FakeExercise::new_with_remainer(10, Operation::Division, 3, 2, 4),
                "10 / 3 = 2 (остаток 4)",
            ),
            (
                FakeExercise::new(12, Operation::Division, 3, 4),
                "12 / 3 = 4",
            ),
        ];

        for (check, expression) in cases {
            assert_eq!(check.evaluate().unwrap(), expression);
        }
    }

    #[test]
    fn test_division_with_remainder() {
        let division = FakeExercise::new_with_remainer(10, Operation::Division, 3, 3, 1);
        assert_eq!(division.evaluate().unwrap(), "10 / 3 = 3 (остаток 1)");
    }

}