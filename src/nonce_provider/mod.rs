mod basic;
mod epoch_provider;

use crate::Result;

trait NonceProvider {
    // Return a (timestamp, nonce) pair.
    fn nonce(&mut self) -> Result<(u32, String)>;
}

pub use epoch_provider::{EpochProvider, SystemEpochProvider};

#[cfg(test)]
pub use epoch_provider::test_provider::TestEpochProvider;