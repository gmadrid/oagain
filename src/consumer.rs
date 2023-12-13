use crate::Result;

#[derive(Debug)]
struct Consumer {
    consumer_key: String,
    consumer_secret: String,
}

impl Consumer {
    pub fn new(key: impl Into<String>, secret: impl Into<String>) -> Result<Consumer> {
        Ok(Consumer {
            consumer_key: key.into(),
            consumer_secret: secret.into(),
        })
    }
}
