use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::convert::From;

/// Like `Option`, but implements `Pod`.
///
/// To ensure that there are no illegal bit patterns or padding bytes,
/// `PodOption` is laid out as a single byte which is 0 in the case of `None`
/// or non-zero in the case of `Some`, and then the value, if any.
#[derive(AnchorSerialize, AnchorDeserialize, Default, Copy, Clone, Debug)]
#[repr(C)]
pub struct PodOption<T: Pod> {
    flag: u64,
    value: T,
}

#[cfg(target_endian = "little")]
unsafe impl<T: Pod> Zeroable for PodOption<T> {}

#[cfg(target_endian = "little")]
unsafe impl<T: Pod> Pod for PodOption<T> {}

impl<T: Pod> From<PodOption<T>> for Option<T> {
    fn from(pod_option: PodOption<T>) -> Self {
        if pod_option.flag > 0 {
            Some(pod_option.value)
        } else {
            None
        }
    }
}

impl<T: Pod> From<Option<T>> for PodOption<T> {
    fn from(normal_option: Option<T>) -> Self {
        match normal_option {
            Some(value) => Self { flag: 1, value },
            None => Self {
                flag: 0,
                value: T::zeroed(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_somes() {
        let pod_option: PodOption<u64> = Some(4).into();
        assert_eq!(Option::from(pod_option), Some(4));

        let pod_option: PodOption<i8> = Some(-123).into();
        assert_eq!(Option::from(pod_option), Some(-123));

        let pod_option: PodOption<Pubkey> = Some(Pubkey::default()).into();
        assert_eq!(Option::from(pod_option), Some(Pubkey::default()));
    }

    #[test]
    pub fn test_nones() {
        let pod_option: PodOption<u64> = None.into();
        assert_eq!(Option::<u64>::from(pod_option), None);

        let pod_option: PodOption<i8> = None.into();
        assert_eq!(Option::<i8>::from(pod_option), None);

        let pod_option: PodOption<Pubkey> = None.into();
        assert_eq!(Option::<Pubkey>::from(pod_option), None);
    }
}
