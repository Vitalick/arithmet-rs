use crate::domain::operation::Operation;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exercise {
    pub left: i32,
    pub right: i32,
    pub operation: Operation,
}


impl Exercise {
    pub fn expected(&self) -> Result<i32, String> {
        self.operation.calculate(self.left, self.right)
    }

    pub fn expected_str(&self) -> Result<String, String> {
        self.operation.calculate_str(self.left, self.right)
    }

    pub fn full_expected(&self) -> Result<String, String> {
        let result = self.expected_str()?;

        Ok(format!("{} = {}", self, result))
    }

    pub fn exercise_for_check(&self, entered: i32) -> Result<[Exercise; 2], String> {

        if self.expected() == Ok(entered) {
            Ok([*self, *self])
        } else {
            Err(format!("Incorrect answer for exercise: {}", self))
        }
    }
}

impl Display for Exercise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.operation, self.right)
    }
}