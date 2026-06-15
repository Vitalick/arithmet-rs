use std::fmt::{Display, Formatter};

pub trait Expression: Display {
    fn evaluate(&self) -> Result<String, String>;

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.evaluate().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::expression::comparison::Comparison;
    use crate::domain::expression::fake_exercise::FakeExercise;
    use crate::domain::operation::Operation;
    use super::*;

    #[test]
    fn test_division_with_remainder() {
        let division = FakeExercise::new_with_remainer(10, Operation::Division, 3, 3, 1);
        assert_eq!(division.evaluate().unwrap(), "10 / 3 = 3 (остаток 1)");
    }

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
