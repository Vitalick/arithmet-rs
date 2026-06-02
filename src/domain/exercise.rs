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

#[cfg(test)]
mod tests {
    use super::*;

    fn checked_expressions(exercise: Exercise, entered: i32) -> Result<Vec<String>, String> {
        exercise
            .exercise_for_check(entered)?
            .into_iter()
            .map(|expression| expression.calculate_expression())
            .collect()
    }

    fn assert_check_error(exercise: Exercise, entered: i32, expected: &str) {
        match exercise.exercise_for_check(entered) {
            Ok(_) => panic!("expected error: {expected}"),
            Err(error) => assert_eq!(error, expected),
        }
    }

    #[test]
    fn test_exercise_calculate_expression_for_each_operation() {
        let cases = [
            (
                Exercise::new(2, Operation::Addition, 3),
                "2 + 3 = 5",
            ),
            (
                Exercise::new(7, Operation::Subtraction, 4),
                "7 - 4 = 3",
            ),
            (
                Exercise::new(6, Operation::Multiplication, 5),
                "6 * 5 = 30",
            ),
            (
                Exercise::new(20, Operation::Division, 4),
                "20 / 4 = 5",
            ),
            (
                Exercise::new(10, Operation::DivisionWithRemainder, 3),
                "10 / 3 = 3 (остаток 1)",
            ),
        ];

        for (exercise, expression) in cases {
            assert_eq!(exercise.calculate_expression().unwrap(), expression);
        }
    }

    #[test]
    fn test_exercise_for_check_addition() {
        assert_eq!(
            checked_expressions(Exercise::new(2, Operation::Addition, 3), 5).unwrap(),
            ["5 - 2 = 3", "5 - 3 = 2"]
        );
    }

    #[test]
    fn test_exercise_for_check_subtraction() {
        assert_eq!(
            checked_expressions(Exercise::new(7, Operation::Subtraction, 4), 3).unwrap(),
            ["7 - 3 = 4", "4 + 3 = 7"]
        );
    }

    #[test]
    fn test_exercise_for_check_multiplication() {
        assert_eq!(
            checked_expressions(Exercise::new(6, Operation::Multiplication, 5), 30).unwrap(),
            ["30 / 6 = 5", "30 / 5 = 6"]
        );
    }

    #[test]
    fn test_exercise_for_check_division() {
        assert_eq!(
            checked_expressions(Exercise::new(20, Operation::Division, 4), 5).unwrap(),
            ["20 / 5 = 4", "4 * 5 = 20"]
        );
    }

    #[test]
    fn test_exercise_for_check_division_with_remainder() {
        assert_eq!(
            checked_expressions(Exercise::new(10, Operation::DivisionWithRemainder, 3), 3).unwrap(),
            ["10 / 3 = 3 (остаток 1)", "1 < 3"]
        );
    }

    #[test]
    fn test_compare_expression() {
        let cases = [
            (Compare::new(1, 3), "1 < 3"),
            (Compare::new(3, 1), "3 > 1"),
            (Compare::new(3, 3), "3 = 3"),
        ];

        for (compare, expression) in cases {
            assert_eq!(compare.calculate_expression().unwrap(), expression);
        }
    }

    #[test]
    fn test_exercise_calculate_expression_with_zero_operands() {
        let cases = [
            (Exercise::new(0, Operation::Addition, 5), "0 + 5 = 5"),
            (Exercise::new(5, Operation::Subtraction, 0), "5 - 0 = 5"),
            (Exercise::new(0, Operation::Multiplication, 5), "0 * 5 = 0"),
            (Exercise::new(0, Operation::Division, 5), "0 / 5 = 0"),
            (
                Exercise::new(0, Operation::DivisionWithRemainder, 5),
                "0 / 5 = 0",
            ),
        ];

        for (exercise, expression) in cases {
            assert_eq!(exercise.calculate_expression().unwrap(), expression);
        }
    }

    #[test]
    fn test_exercise_rejects_division_by_zero() {
        let division = Exercise::new(5, Operation::Division, 0);
        let division_with_remainder = Exercise::new(5, Operation::DivisionWithRemainder, 0);

        assert_eq!(
            division.calculate_expression().unwrap_err(),
            "Деление на ноль"
        );
        assert_eq!(
            division_with_remainder.calculate_expression().unwrap_err(),
            "Деление на ноль"
        );
    }

    #[test]
    fn test_exercise_for_check_rejects_zero_answer_for_each_operation() {
        let cases = [
            Exercise::new(2, Operation::Addition, 3),
            Exercise::new(7, Operation::Subtraction, 4),
            Exercise::new(6, Operation::Multiplication, 5),
            Exercise::new(20, Operation::Division, 4),
            Exercise::new(10, Operation::DivisionWithRemainder, 3),
        ];

        for exercise in cases {
            assert_check_error(exercise, 0, "Ответ не должен быть нулём");
        }
    }

    #[test]
    fn test_exercise_for_check_rejects_source_division_by_zero() {
        let cases = [
            Exercise::new(5, Operation::Division, 0),
            Exercise::new(5, Operation::DivisionWithRemainder, 0),
        ];

        for exercise in cases {
            assert_check_error(exercise, 1, "Деление на ноль");
        }
    }
}
