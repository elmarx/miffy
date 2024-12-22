use serde::{Serialize, Serializer};

#[derive(Serialize)]
struct Error<E>
where
    E: Serialize,
{
    error: E,
}

/// custom serializer for Result<T, E> that serializes the error as a separate field
pub fn custom_result<S, T, E>(value: &Result<T, E>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
    E: Serialize,
{
    match value {
        Ok(t) => t.serialize(s),
        Err(e) => Error { error: e }.serialize(s),
    }
}

#[cfg(test)]
mod test {
    use serde::Serialize;
    use serde_json::json;

    #[derive(Serialize)]
    struct Example {
        #[serde(serialize_with = "super::custom_result")]
        result: Result<String, String>,
    }

    #[test]
    fn serialize_result_ok() {
        let sample = Example {
            result: Ok("success".to_string()),
        };

        let actual = serde_json::to_value(&sample).expect("ok");
        assert_eq!(actual, json! { {"result": "success"} });
    }

    #[test]
    fn serialize_result_err() {
        let sample = Example {
            result: Err("failure".to_string()),
        };

        let actual = serde_json::to_value(&sample).expect("ok");
        assert_eq!(actual, json! { {"result": {"error": "failure"}} });
    }
}
