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
}