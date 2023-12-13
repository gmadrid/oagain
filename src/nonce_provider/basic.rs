use crate::nonce_provider::epoch_provider::{EpochProvider, SystemEpochProvider};
use crate::nonce_provider::NonceProvider;
use crate::Result;
use std::collections::HashSet;

// TODO: you could add a nonce_generator to make the nonce value random.

#[derive(Debug)]
struct BasicNonce<EP: EpochProvider> {
    epoch_provider: EP,
    last_timestamp: u32,
    nonces_for_last_timestamp: HashSet<String>,
}

impl Default for BasicNonce<SystemEpochProvider> {
    fn default() -> Self {
        BasicNonce::<SystemEpochProvider>::new(SystemEpochProvider)
    }
}

impl<EP: EpochProvider> BasicNonce<EP> {
    fn new(epoch_provider: EP) -> BasicNonce<EP> {
        BasicNonce {
            epoch_provider,
            last_timestamp: Default::default(),
            nonces_for_last_timestamp: Default::default(),
        }
    }

    #[cfg(test)]
    fn epoch_provider(&mut self) -> &mut EP {
        &mut self.epoch_provider
    }
}

impl<EP: EpochProvider> NonceProvider for BasicNonce<EP> {
    fn nonce(&mut self) -> Result<(u32, String)> {
        // let now = std::time::SystemTime::now();
        // let epoch_duration = now.duration_since(UNIX_EPOCH)?;
        //
        // // Should fit. At least until 2033
        // let epoch_millis = epoch_duration.as_millis() as u32;
        let epoch_millis = self.epoch_provider.epoch();
        if epoch_millis == self.last_timestamp {
            let nonce = format!(
                "nonce-{}-{}",
                epoch_millis,
                self.nonces_for_last_timestamp.len()
            );
            self.nonces_for_last_timestamp.insert(nonce.clone());
            Ok((epoch_millis, nonce))
        } else {
            self.nonces_for_last_timestamp.clear();
            self.last_timestamp = epoch_millis;
            let nonce = format!("nonce-{}-0", epoch_millis);
            self.nonces_for_last_timestamp.insert(nonce.clone());
            Ok((epoch_millis, nonce))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::nonce_provider::basic::BasicNonce;
    use crate::nonce_provider::epoch_provider::test_provider::TestEpochProvider;
    use crate::nonce_provider::NonceProvider;

    #[test]
    fn basic() {
        let mut epoch_provider = TestEpochProvider::new(44);
        let mut nonce_provider = BasicNonce::new(epoch_provider);

        assert_eq!(
            (44, "nonce-44-0".to_string()),
            nonce_provider.nonce().unwrap()
        );
        assert_eq!(
            (44, "nonce-44-1".to_string()),
            nonce_provider.nonce().unwrap()
        );
        assert_eq!(
            (44, "nonce-44-2".to_string()),
            nonce_provider.nonce().unwrap()
        );

        nonce_provider.epoch_provider().new_value(88);
        assert_eq!(
            (88, "nonce-88-0".to_string()),
            nonce_provider.nonce().unwrap()
        );
        assert_eq!(
            (88, "nonce-88-1".to_string()),
            nonce_provider.nonce().unwrap()
        );
        assert_eq!(
            (88, "nonce-88-2".to_string()),
            nonce_provider.nonce().unwrap()
        );
    }
}
