use std::fmt::Display;
use crate::domain::exercise::Exercise;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Addition,              // сложение
    Subtraction,           // вычитание
    Multiplication,        // умножение
    Division,              // деление
    DivisionWithRemainder, // деление с остатком
    // RemainderFromDivision, // остаток от деления
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Addition => write!(f, "+"),
            Operation::Subtraction => write!(f, "-"),
            Operation::Multiplication => write!(f, "*"),
            Operation::Division|Operation::DivisionWithRemainder => write!(f, "/"),
        }
    }
}

impl From<char> for Operation {
    fn from(c: char) -> Self {
        match c {
            '+' => Operation::Addition,
            '-' => Operation::Subtraction,
            '*' => Operation::Multiplication,
            '/' => Operation::Division,
            ':' => Operation::DivisionWithRemainder,
            _ => panic!("Некорректная операция: {}", c),
        }
    }
}

impl From<&str> for Operation {
    fn from(s: &str) -> Self {
        s.chars().next().unwrap().into()
    }
}

impl Operation {
    pub fn hotkey_str(&self) -> String {
        match self {
            Operation::DivisionWithRemainder => ":".to_string(),
            _ => format!("{}", self)
        }
    }

    pub fn make_exercise(&self, left: i32, right: i32) -> Exercise {
         Exercise::new(left, *self, right)
    }

    pub fn validates_operands(&self, left: i32, right: i32) -> Result<(), String> {
        match self {
            Operation::Division | Operation::DivisionWithRemainder => {
                if right != 0 {
                    return Ok(());
                }
                Err("Деление на ноль".to_string())
            }
            _ => Ok(()),
        }
    }

    pub fn calculate(&self, left: i32, right: i32) -> Result<i32, String> {
        self.validates_operands(left, right)?;

        match self {
            Operation::Addition => Ok(left + right),
            Operation::Subtraction => Ok(left - right),
            Operation::Multiplication => Ok(left * right),
            Operation::Division => Ok(left / right),
            Operation::DivisionWithRemainder => Ok(left / right),
        }
    }

    pub fn calculate_str(&self, left: i32, right: i32) -> Result<String, String> {
        self.validates_operands(left, right)?;

        let result = self.calculate(left, right)?;
        if matches!(self, Operation::Division | Operation::DivisionWithRemainder) {
            let reminder = left % right;
            if reminder != 0 {
                return Ok(format!("{} (остаток {})", result, reminder));
            }
        }
        Ok(format!("{}", result))
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_from_char() {
        assert_eq!(Operation::Addition, '+'.into());
        assert_eq!(Operation::Subtraction, '-'.into());
        assert_eq!(Operation::Multiplication, '*'.into());
        assert_eq!(Operation::Division, '/'.into());
        assert_eq!(Operation::DivisionWithRemainder, ':'.into());
    }

    #[test]
    fn test_calculate() {
        assert_eq!(Operation::Addition.calculate(1, 2).unwrap(), 3);
        assert_eq!(Operation::Subtraction.calculate(5, 3).unwrap(), 2);
        assert_eq!(Operation::Multiplication.calculate(4, 6).unwrap(), 24);
        assert_eq!(Operation::Division.calculate(10, 2).unwrap(), 5);
        assert_eq!(Operation::DivisionWithRemainder.calculate(10, 3).unwrap(), 3);
    }

    #[test]
    fn test_calculate_str() {
        assert_eq!(Operation::Addition.calculate_str(1, 2).unwrap(), "3");
        assert_eq!(Operation::Subtraction.calculate_str(5, 3).unwrap(), "2");
        assert_eq!(Operation::Multiplication.calculate_str(4, 6).unwrap(), "24");
        assert_eq!(Operation::Division.calculate_str(10, 2).unwrap(), "5");
        assert_eq!(Operation::DivisionWithRemainder.calculate_str(10, 3).unwrap(), "3 (остаток 1)");
    }

    #[test]
    fn test_validate_operands() {
        assert!(Operation::Addition.validates_operands(1, 2).is_ok());
        assert!(Operation::Subtraction.validates_operands(5, 3).is_ok());
        assert!(Operation::Multiplication.validates_operands(4, 6).is_ok());
        assert!(Operation::Division.validates_operands(10, 2).is_ok());
        assert!(Operation::DivisionWithRemainder.validates_operands(10, 3).is_ok());
        assert!(Operation::Division.validates_operands(10, 0).is_err());
    }
}