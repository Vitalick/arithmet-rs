use crate::domain::exercise::Exercise;
use crate::domain::operation::Operation;
use std::fmt::{Display, Formatter};

pub trait Expression: Display {
    fn evaluate(&self) -> Result<String, String>;

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
pub struct FakeExercise {
    pub left: i32,
    pub operation: Operation,
    pub right: i32,
    pub answer: i32,
    pub remainder: i32,
}

impl FakeExercise {
    pub fn new(left: i32, operation: Operation, right: i32, answer: i32) -> Self {
        FakeExercise {
            left,
            operation,
            right,
            answer,
            remainder: 0,
        }
    }

    pub fn new_with_remainer(
        left: i32,
        operation: Operation,
        right: i32,
        answer: i32,
        remainder: i32,
    ) -> Self {
        FakeExercise {
            left,
            operation,
            right,
            answer,
            remainder,
        }
    }

    pub fn with_remainder(self, remainder: i32) -> Self {
        FakeExercise { remainder, ..self }
    }

    pub fn with_answer(self, answer: i32) -> Self {
        FakeExercise { answer, ..self }
    }

    pub fn clone_exercise(exercise: Exercise) -> Result<Self, String> {
        Ok(FakeExercise {
            left: exercise.left,
            operation: exercise.operation,
            right: exercise.right,
            answer: exercise.expected()?,
            remainder: exercise.expected_remainder()?,
        })
    }
}

impl From<Exercise> for FakeExercise {
    fn from(exercise: Exercise) -> Self {
        FakeExercise::clone_exercise(exercise).unwrap()
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
    fn test_division_with_remainder() {
        let division = FakeExercise::new_with_remainer(10, Operation::Division, 3, 3, 1);
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
}
