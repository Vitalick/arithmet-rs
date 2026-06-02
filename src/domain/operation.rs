use std::fmt::Display;

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
            Operation::Division => write!(f, "/"),
            Operation::DivisionWithRemainder => write!(f, ":"),
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
            _ => panic!("Invalid operation: {}", c),
        }
    }
}

impl From<&str> for Operation {
    fn from(s: &str) -> Self {
        s.chars().next().unwrap().into()
    }
}

impl Operation {

    pub fn validates_operands(&self, a: i32, b: i32) -> Result<bool, String> {
        match self {
            Operation::Division | Operation::DivisionWithRemainder => {
                if b != 0 {
                    return Ok(true);
                }
                Err("Division by zero".to_string())
            }
            _ => Ok(true),
        }
    }

    pub fn calculate(&self, a: i32, b: i32) -> Result<i32, String> {
        self.validates_operands(a, b)?;

        match self {
            Operation::Addition => Ok(a + b),
            Operation::Subtraction => Ok(a - b),
            Operation::Multiplication => Ok(a * b),
            Operation::Division => Ok(a / b),
            Operation::DivisionWithRemainder => Ok(a / b),
        }
    }

    pub fn calculate_str(&self, a: i32, b: i32) -> Result<String, String> {
        self.validates_operands(a, b)?;

        let result = self.calculate(a, b)?;
        if matches!(self, Operation::Division | Operation::DivisionWithRemainder) {
            let reminder = a % b;
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
        assert!(Operation::Addition.validates_operands(1, 2).unwrap());
        assert!(Operation::Subtraction.validates_operands(5, 3).unwrap());
        assert!(Operation::Multiplication.validates_operands(4, 6).unwrap());
        assert!(Operation::Division.validates_operands(10, 2).unwrap());
        assert!(Operation::DivisionWithRemainder.validates_operands(10, 3).unwrap());
        assert!(Operation::Division.validates_operands(10, 0).is_err());
    }
}