use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    // сложение
    Addition,
    // вычитание
    Subtraction,
    // умножение
    Multiplication,
    // деление
    Division,
    // деление с остатком
    DivisionWithRemainder,
    // остаток от деления
    // RemainderFromDivision,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Addition => write!(f, "+"),
            Operation::Subtraction => write!(f, "-"),
            Operation::Multiplication => write!(f, "*"),
            Operation::Division | Operation::DivisionWithRemainder => write!(f, "/"),
        }
    }
}

impl Operation {
    pub fn label(&self) -> &'static str {
        match self {
            Operation::Addition => "Сложение",
            Operation::Subtraction => "Вычитание",
            Operation::Multiplication => "Умножение",
            Operation::Division => "Деление",
            Operation::DivisionWithRemainder => "Деление с остатком",
        }
    }

    fn from_symbol(symbol: &str) -> Result<Self, String> {
        match symbol {
            "+" => Ok(Operation::Addition),
            "-" => Ok(Operation::Subtraction),
            "*" => Ok(Operation::Multiplication),
            "/" => Ok(Operation::Division),
            ":" => Ok(Operation::DivisionWithRemainder),
            _ => Err(format!("Некорректная операция: {}", symbol)),
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Operation::Addition => "+",
            Operation::Subtraction => "-",
            Operation::Multiplication => "*",
            Operation::Division => "/",
            Operation::DivisionWithRemainder => ":",
        }
    }
}

impl Serialize for Operation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.symbol())
    }
}

impl<'de> Deserialize<'de> for Operation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let symbol = String::deserialize(deserializer)?;
        Operation::from_symbol(&symbol).map_err(serde::de::Error::custom)
    }
}

impl From<char> for Operation {
    fn from(c: char) -> Self {
        Operation::from_symbol(&c.to_string()).unwrap()
    }
}

impl From<&str> for Operation {
    fn from(s: &str) -> Self {
        Operation::from_symbol(s).unwrap()
    }
}

impl Operation {
    pub fn hotkey_str(&self) -> String {
        self.symbol().to_string()
    }

    pub fn validates_operands(&self, _left: i32, right: i32) -> Result<(), String> {
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

    pub fn calculate_remainder(&self, left: i32, right: i32) -> Result<i32, String> {
        self.validates_operands(left, right)?;

        if matches!(self, Operation::Division | Operation::DivisionWithRemainder) {
            return Ok(left % right);
        }
        Ok(0)
    }

    pub fn calculate_str(&self, left: i32, right: i32) -> Result<String, String> {
        self.validates_operands(left, right)?;

        let result = self.calculate(left, right)?;
        let remainder = self.calculate_remainder(left, right)?;
        if remainder != 0 {
            return Ok(format!("{} (остаток {})", result, remainder));
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
        assert_eq!(
            Operation::DivisionWithRemainder.calculate(10, 3).unwrap(),
            3
        );
    }

    #[test]
    fn test_calculate_str() {
        assert_eq!(Operation::Addition.calculate_str(1, 2).unwrap(), "3");
        assert_eq!(Operation::Subtraction.calculate_str(5, 3).unwrap(), "2");
        assert_eq!(Operation::Multiplication.calculate_str(4, 6).unwrap(), "24");
        assert_eq!(Operation::Division.calculate_str(10, 2).unwrap(), "5");
        assert_eq!(
            Operation::DivisionWithRemainder
                .calculate_str(10, 3)
                .unwrap(),
            "3 (остаток 1)"
        );
    }

    #[test]
    fn test_validate_operands() {
        assert!(Operation::Addition.validates_operands(1, 2).is_ok());
        assert!(Operation::Subtraction.validates_operands(5, 3).is_ok());
        assert!(Operation::Multiplication.validates_operands(4, 6).is_ok());
        assert!(Operation::Division.validates_operands(10, 2).is_ok());
        assert!(Operation::DivisionWithRemainder
            .validates_operands(10, 3)
            .is_ok());
        assert!(Operation::Division.validates_operands(10, 0).is_err());
    }
}
