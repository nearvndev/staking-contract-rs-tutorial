use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub reward_numerator: u32,
    pub reward_denumerator: u64
}

impl Default for Config {
    fn default() -> Self {
        // APR 15% - 18%
        Self { reward_numerator: 715, reward_denumerator: 100000000000 } // reward per block
    }
}