use crate::error::OpenBookError;
use anchor_lang::prelude::*;

#[macro_export]
macro_rules! zip {
    ($x: expr) => ($x);
    ($x: expr, $($y: expr), +) => (
        $x.zip(
            zip!($($y), +))
    )
}
#[allow(unused_imports)]
pub(crate) use zip;

pub fn fill_from_str<const N: usize>(name: &str) -> Result<[u8; N]> {
    let name_bytes = name.as_bytes();
    require!(name_bytes.len() <= N, OpenBookError::SomeError);
    let mut name_ = [0u8; N];
    name_[..name_bytes.len()].copy_from_slice(name_bytes);
    Ok(name_)
}

pub fn format_zero_terminated_utf8_bytes(
    name: &[u8],
    fmt: &mut std::fmt::Formatter,
) -> std::result::Result<(), std::fmt::Error> {
    fmt.write_str(
        std::str::from_utf8(name)
            .unwrap()
            .trim_matches(char::from(0)),
    )
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
