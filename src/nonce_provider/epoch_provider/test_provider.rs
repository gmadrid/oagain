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
