#[cfg(test)]
pub mod test_provider;

use std::time::UNIX_EPOCH;

pub trait EpochProvider {
    fn epoch(&self) -> u32;
}

pub struct SystemEpochProvider;

impl EpochProvider for SystemEpochProvider {
    fn epoch(&self) -> u32 {
        let now = std::time::SystemTime::now();
        let epoch_duration = now.duration_since(UNIX_EPOCH).unwrap();

        // Should fit. At least until 2038
        epoch_duration.as_secs() as u32
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
            .as_secs() as u32;

        dbg!(time);
        dbg!(millis);

        // There is no really good way to test_cases this. We just ensure that the time returned by
        // provider is within 10ms of the current time expressed as ms from the unix epoch.
        assert!(millis.abs_diff(time) < 10);
    }
}
