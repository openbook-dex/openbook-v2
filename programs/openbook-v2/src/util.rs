use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn fill_from_str<const N: usize>(name: &str) -> Result<[u8; N]> {
    let name_bytes = name.as_bytes();
    require!(name_bytes.len() <= N, OpenBookError::InvalidInputNameLength);
    let mut name_ = [0u8; N];
    name_[..name_bytes.len()].copy_from_slice(name_bytes);
    Ok(name_)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_from_str() {
        assert_eq!(fill_from_str::<4>(""), Ok([0, 0, 0, 0]));
        assert_eq!(fill_from_str::<4>("abc"), Ok([b'a', b'b', b'c', 0]));
        assert_eq!(fill_from_str::<4>("abcd"), Ok([b'a', b'b', b'c', b'd']));
        assert!(fill_from_str::<4>("abcde").is_err());
    }
}
