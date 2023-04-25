use super::decimal::SwitchboardDecimal;
use super::error::SwitchboardError;
use anchor_lang::prelude::*;
use bytemuck::{try_cast_slice, try_from_bytes};
use bytemuck::{Pod, Zeroable};
use std::cell::Ref;
use superslice::*;

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct AggregatorHistoryRow {
    /// The timestamp of the sample.
    pub timestamp: i64,
    /// The value of the sample.
    pub value: SwitchboardDecimal,
}
unsafe impl Pod for AggregatorHistoryRow {}
unsafe impl Zeroable for AggregatorHistoryRow {}

pub struct AggregatorHistoryBuffer<'a> {
    /// The current index of the round robin buffer.
    pub insertion_idx: usize,
    /// The array of samples collected from the aggregator.
    pub rows: Ref<'a, [AggregatorHistoryRow]>,
}
impl<'a> AggregatorHistoryBuffer<'a> {
    /// Returns the deserialized Switchboard history buffer account
    ///
    /// # Arguments
    ///
    /// * `history_buffer` - A Solana AccountInfo referencing an existing Switchboard history buffer account
    pub fn new(
        history_buffer: &'a AccountInfo,
    ) -> anchor_lang::Result<AggregatorHistoryBuffer<'a>> {
        let data = history_buffer.try_borrow_data()?;

        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != *b"BUFFERxx" {
            return Err(SwitchboardError::AccountDiscriminatorMismatch.into());
        }
        let insertion_idx: u32 = try_from_bytes::<u32>(&data[8..12]).unwrap().clone();
        let rows = Ref::map(data, |data| try_cast_slice(&data[12..]).unwrap());
        return Ok(Self {
            insertion_idx: insertion_idx as usize,
            rows: rows,
        });
    }

    /// Return the previous row in the history buffer for a given timestamp
    ///
    /// # Arguments
    ///
    /// * `timestamp` - A unix timestamp to search in the history buffer
    pub fn lower_bound(&self, timestamp: i64) -> Option<AggregatorHistoryRow> {
        if self.rows[self.insertion_idx].timestamp == 0 {
            return None;
        }
        let lower = &self.rows[..self.insertion_idx + 1];
        let lahr = lower.lower_bound_by(|x| {
            let other: i64 = x.timestamp;
            other.cmp(&timestamp)
        });
        if lahr < lower.len() && lower[lahr].timestamp == timestamp {
            return Some(lower[lahr]);
        }
        if lahr != 0 {
            return Some(lower[lahr - 1]);
        }

        if self.insertion_idx + 1 < self.rows.len()
            && self.rows[self.insertion_idx + 1].timestamp != 0
        {
            let upper = &self.rows[self.insertion_idx + 1..];
            let uahr = upper.lower_bound_by(|x| {
                let other: i64 = x.timestamp;
                other.cmp(&timestamp)
            });
            if uahr < upper.len() && upper[uahr].timestamp == timestamp {
                return Some(upper[uahr]);
            }
            if uahr != 0 {
                return Some(upper[uahr - 1]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    impl<'info, 'a> Default for AggregatorHistoryBuffer<'a> {
        fn default() -> Self {
            unsafe { std::mem::zeroed() }
        }
    }

    // insertion_idx = 1
    // 1646249940   - 100.6022611525
    // 1646249949   - 100.5200735
    // 1646249713   - 100.3012875
    // 1646249752   - 100.495469495
    // 1646249881   - 100.5763445
    // 1646249893   - 100.4691257925
    // 1646249902   - 100.517196115
    // 1646249911   - 100.5026458225
    // 1646249918   - 100.52034706
    // 1646249929   - 100.6000855

    const HISTORY_BUFFER_DATA: [u8; 292] = [
        66, 85, 70, 70, 69, 82, 120, 120, 1, 0, 0, 0, 212, 199, 31, 98, 0, 0, 0, 0, 69, 210, 158,
        59, 234, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 221, 199, 31, 98, 0, 0, 0, 0, 95,
        37, 234, 59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 241, 198, 31, 98, 0, 0, 0, 0,
        11, 195, 200, 59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 24, 199, 31, 98, 0, 0, 0,
        0, 183, 43, 255, 101, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 153, 199, 31, 98, 0,
        0, 0, 0, 117, 187, 242, 59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 165, 199, 31,
        98, 0, 0, 0, 0, 69, 250, 67, 236, 233, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 174,
        199, 31, 98, 0, 0, 0, 0, 83, 177, 74, 103, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0,
        183, 199, 31, 98, 0, 0, 0, 0, 113, 186, 62, 0, 234, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0,
        0, 0, 190, 199, 31, 98, 0, 0, 0, 0, 146, 224, 37, 87, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        8, 0, 0, 0, 201, 199, 31, 98, 0, 0, 0, 0, 215, 90, 246, 59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 7, 0, 0, 0,
    ];

    // DiFWjRtc9PQGposykEULC93y7uTXde3Eyr7HnQ7kvqkD
    const HISTORY_BUFFER_PUBKEY: Pubkey = Pubkey::new_from_array([
        188, 221, 119, 59, 130, 153, 226, 148, 95, 158, 33, 63, 106, 233, 240, 46, 242, 141, 150,
        147, 148, 158, 88, 14, 59, 66, 18, 82, 181, 250, 102, 130,
    ]);

    #[test]
    fn test_history_buffer() {
        let mut history_data = HISTORY_BUFFER_DATA.clone();
        let mut lamports = 0;
        let history_account_info = AccountInfo::new(
            &HISTORY_BUFFER_PUBKEY,
            false,
            false,
            &mut lamports,
            &mut history_data,
            &SWITCHBOARD_V2_DEVNET,
            false,
            0,
        );
        let history_buffer = AggregatorHistoryBuffer::new(&history_account_info).unwrap();

        // let mut counter = 0;
        // for row in history_buffer.rows.iter() {
        //     let val: f64 = row.value.try_into().unwrap();
        //     println!(
        //         "[{}] {} - {:?} = {}",
        //         counter, row.timestamp, row.value, val
        //     );
        //     counter = counter + 1;
        // }

        // Get result at exact timestamp, lower bound
        match history_buffer.lower_bound(1646249940) {
            None => panic!("failed to retrieve a value for a valid timestamp"),
            Some(row) => {
                let correct_value = SwitchboardDecimal {
                    mantissa: 1006022611525,
                    scale: 10,
                };
                if row.value != correct_value {
                    panic!(
                        "failed to retrieve correct value at exact timestamp 1646249940. received: {:?}, expected: {:?}",
                        row.value, correct_value
                    )
                }
            }
        };

        // Get result at exact timestamp, lower bound
        match history_buffer.lower_bound(1646249949) {
            None => panic!("failed to retrieve a value for a valid timestamp"),
            Some(row) => {
                let correct_value = SwitchboardDecimal {
                    mantissa: 1005200735,
                    scale: 7,
                };
                if row.value != correct_value {
                    panic!(
                        "failed to retrieve correct value at exact timestamp 1646249940. received: {:?}, expected: {:?}",
                        row.value, correct_value
                    )
                }
            }
        };

        // Get result at exact timestamp, upper bound
        match history_buffer.lower_bound(1646249911) {
            None => panic!("failed to retrieve a value for a valid timestamp"),
            Some(row) => {
                let correct_value = SwitchboardDecimal {
                    mantissa: 1005026458225,
                    scale: 10,
                };
                if row.value != correct_value {
                    panic!(
                        "failed to retrieve correct value at exact timestamp 1646249911. received: {:?}, expected: {:?}",
                        row.value, correct_value
                    )
                }
            }
        };

        // Get result at exact timestamp, upper bound
        match history_buffer.lower_bound(1646249929) {
            None => panic!("failed to retrieve a value for a valid timestamp"),
            Some(row) => {
                let correct_value = SwitchboardDecimal {
                    mantissa: 1006000855,
                    scale: 7,
                };
                if row.value != correct_value {
                    panic!(
                        "failed to retrieve correct value at exact timestamp 1646249911. received: {:?}, expected: {:?}",
                        row.value, correct_value
                    )
                }
            }
        };

        // Get lower bound result
        match history_buffer.lower_bound(1646249912) {
            None => panic!("failed to retrieve a value for a valid timestamp"),
            Some(row) => {
                let correct_value = SwitchboardDecimal {
                    mantissa: 1005026458225,
                    scale: 10,
                };
                if row.value != correct_value {
                    panic!("failed to retrieve correct value for timestamp 1646249912. received: {:?}, expected: {:?}",row.value, correct_value)
                }
            }
        };

        // Get previous result
        match history_buffer.lower_bound(1646249910) {
            None => panic!("failed to retrieve a value for a valid timestamp"),
            Some(row) => {
                let correct_value = SwitchboardDecimal {
                    mantissa: 100517196115,
                    scale: 9,
                };
                if row.value != correct_value {
                    panic!(
                        "failed to retrieve correct value for timestamp 1646249910. received: {:?}, expected: {:?}",
                        row.value, correct_value
                    )
                }
            }
        };

        // Get future result
        match history_buffer.lower_bound(2646249911) {
            None => panic!("failed to retrieve a value for a valid timestamp"),
            Some(row) => {
                let correct_value = SwitchboardDecimal {
                    mantissa: 1005200735,
                    scale: 7,
                };
                if row.value != correct_value {
                    panic!("failed to retrieve correct value for timestamp 2646249911. received: {:?}, expected: {:?}",row.value, correct_value)
                }
            }
        };

        // Get past result
        match history_buffer.lower_bound(0646249911) {
            None => (),
            Some(row) => panic!("retrieved row when no value was expected {:?}", row.value),
        };
    }
}
