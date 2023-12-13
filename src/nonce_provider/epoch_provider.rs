use std::time::UNIX_EPOCH;

pub trait EpochProvider {
    fn epoch(&self) -> u32;
}

pub struct SystemEpochProvider;

impl EpochProvider for SystemEpochProvider {
    fn epoch(&self) -> u32 {
        let now = std::time::SystemTime::now();
        let epoch_duration = now.duration_since(UNIX_EPOCH).unwrap();

        // Should fit. At least until 2033
        epoch_duration.as_millis() as u32
    }
}

#[cfg(test)]
pub mod test_provider {
    use crate::nonce_provider::epoch_provider::EpochProvider;

    pub struct TestEpochProvider(u32);

    impl TestEpochProvider {
        pub fn new(val: u32) -> TestEpochProvider {
            TestEpochProvider(val)
        }

        pub fn new_value(&mut self, value: u32) {
            self.0 = value;
        }
    }

    impl EpochProvider for TestEpochProvider {
        fn epoch(&self) -> u32 {
            self.0
        }
    }

    mod test {
        use crate::nonce_provider::epoch_provider::test_provider::TestEpochProvider;
        use crate::nonce_provider::epoch_provider::EpochProvider;

        #[test]
        fn test_provider_test() {
            let mut provider = TestEpochProvider::new(42);
            assert_eq!(42, provider.epoch());
            assert_eq!(42, provider.epoch());
            assert_eq!(42, provider.epoch());

            provider.new_value(84);
            assert_eq!(84, provider.epoch());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn basic() {
        let epoch_provider = SystemEpochProvider;

        let time = epoch_provider.epoch();
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        // There is no really good way to test this. We just ensure that the time returned by
        // provider is within 10ms of the current time expressed as ms from the unix epoch.
        assert!(millis.abs_diff(time) < 10);
    }
}
