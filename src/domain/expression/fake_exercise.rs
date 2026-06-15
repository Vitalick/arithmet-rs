use std::fmt::{Display, Formatter};
use crate::domain::expression::Expression;
use crate::domain::operation::Operation;

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