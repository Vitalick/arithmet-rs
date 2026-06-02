use crate::domain::operation::Operation;
use std::fmt::{Display, Formatter};

pub trait CalculableExpression {
    fn calculate_expression(&self) -> Result<String, String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Compare {
    pub left: i32,
    pub right: i32,
}

impl Compare {
    pub fn new(left: i32, right: i32) -> Self {
        Compare {
            left,
            right,
        }
    }
}

impl Display for Compare {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ? {}", self.left, self.right)
    }
}

impl CalculableExpression for Compare {
    fn calculate_expression(&self) -> Result<String, String> {
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
pub struct Exercise {
    pub left: i32,
    pub operation: Operation,
    pub right: i32,
}

impl CalculableExpression for Exercise {
    fn calculate_expression(&self) -> Result<String, String> {
        let result = self.expected_str()?;

        Ok(format!("{} = {}", self, result))
    }
}

impl Exercise {
    pub fn new(left: i32, operation: Operation, right: i32) -> Self {
        Exercise { left, operation, right }
    }

    pub fn expected(&self) -> Result<i32, String> {
        self.operation.calculate(self.left, self.right)
    }

    pub fn expected_str(&self) -> Result<String, String> {
        self.operation.calculate_str(self.left, self.right)
    }

    fn validate_operands(&self) -> Result<(), String> {
        self.operation.validates_operands(self.left, self.right)
    }

    pub fn exercise_for_check(
        &self,
        entered: i32,
    ) -> Result<[Box<dyn CalculableExpression>; 2], String> {
        if entered == 0 {
            return Err("Ответ не должен быть нулём".to_string());
        }
        self.validate_operands()?;
        match self.operation {
            Operation::Addition => Ok([
                Box::new(Self::new(entered, Operation::Subtraction, self.left)),
                Box::new(Self::new(entered, Operation::Subtraction, self.right)),
            ]),
            Operation::Subtraction => Ok([
                Box::new(Self::new(self.left, Operation::Subtraction, entered)),
                Box::new(Self::new(self.right, Operation::Addition, entered)),
            ]),
            Operation::Multiplication => Ok([
                Box::new(Self::new(entered, Operation::Division, self.left)),
                Box::new(Self::new(entered, Operation::Division, self.right)),
            ]),
            Operation::Division => Ok([
                Box::new(Self::new(self.left, Operation::Division, entered)),
                Box::new(Self::new(self.right, Operation::Multiplication, entered)),
            ]),
            Operation::DivisionWithRemainder => Ok([
                Box::new(*self),
                Box::new(Compare::new(self.left % self.right, self.right)),
            ]),
        }
    }
}

impl Display for Exercise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.operation, self.right)
    }
}
