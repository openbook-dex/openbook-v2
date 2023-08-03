use fixed::types::I80F48;

pub trait Power {
    fn square(&self) -> Self;
    fn checked_square(&self) -> Option<Self>
    where
        Self: Sized;
}

impl Power for I80F48 {
    fn square(&self) -> Self {
        self * self
    }

    fn checked_square(&self) -> Option<Self> {
        self.checked_mul(*self)
    }
}
