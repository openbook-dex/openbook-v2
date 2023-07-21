use fixed::types::I80F48;

pub trait Power {
    fn square(&self) -> Self;
}

impl Power for I80F48 {
    fn square(&self) -> Self {
        self * self
    }
}
