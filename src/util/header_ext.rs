use http::HeaderValue;

pub trait TxHeader {
    fn is_json(&self) -> bool;
}

impl TxHeader for HeaderValue {
    fn is_json(&self) -> bool {
        self == HeaderValue::from_static("application/json")
            || (self
                .to_str()
                .is_ok_and(|s| s.starts_with("application/") && s.ends_with("+json")))
    }
}

#[cfg(test)]
mod test {
    use super::TxHeader;
    use http::HeaderValue;

    #[test]
    fn test_default() {
        let sample = HeaderValue::from_static("application/json");

        assert!(sample.is_json());
    }

    #[test]
    fn test_text() {
        let sample = HeaderValue::from_static("text/plain");

        assert!(!sample.is_json());
    }

    #[test]
    fn test_extension() {
        let sample = HeaderValue::from_static("application/vnd.spring-boot.actuator.v3+json");

        assert!(sample.is_json());
    }
}
