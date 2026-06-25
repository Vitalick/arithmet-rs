use std::fmt::{Display, Formatter};

pub trait Expression: Display {
    fn evaluate(&self) -> Result<String, String>;

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.evaluate().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::expression::fake_exercise::FakeExercise;
    use crate::domain::operation::Operation;
}
