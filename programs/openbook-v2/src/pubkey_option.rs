use anchor_lang::prelude::*;
use bytemuck::Zeroable;
use std::convert::From;

/// Like `Option`, but implemented for `Pubkey`.
#[zero_copy]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct NonZeroPubkeyOption {
    key: Pubkey,
}

impl From<NonZeroPubkeyOption> for Option<Pubkey> {
    fn from(pubkey_option: NonZeroPubkeyOption) -> Self {
        if pubkey_option.key == Pubkey::zeroed() {
            None
        } else {
            Some(pubkey_option.key)
        }
    }
}

impl From<Option<Pubkey>> for NonZeroPubkeyOption {
    fn from(normal_option: Option<Pubkey>) -> Self {
        match normal_option {
            Some(key) => Self { key },
            None => Self::zeroed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_some() {
        let pubkey_option: NonZeroPubkeyOption = Some(crate::ID).into();
        assert_eq!(Option::<Pubkey>::from(pubkey_option), Some(crate::ID));
    }

    #[test]
    pub fn test_none() {
        let pubkey_option: NonZeroPubkeyOption = None.into();
        assert_eq!(Option::<Pubkey>::from(pubkey_option), None);

        // the default pubkey also matches none
        assert_eq!(Pubkey::default(), Pubkey::zeroed());
        let pubkey_option: NonZeroPubkeyOption = Some(Pubkey::default()).into();
        assert_eq!(Option::<Pubkey>::from(pubkey_option), None);
    }
}
