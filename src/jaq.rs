use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Ctx, Filter, Native, RcIter};
use jaq_json::Val;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed compiling JAQ filter")]
    Compile,
    #[error("failed loading JAQ filter `{}`", .0)]
    Load(String),
    #[error("filter produced no output")]
    EmptyOutput,
    #[error("filter produced invalid output")]
    ExtractingOutput,

    #[error("Filter producet mutiple outputs")]
    MultipleOutputs,
}

/// JSON transformer
///
/// based on JAQ (i.e.: JQ). A naive wrapper to simplify miffy's use-case.
pub struct JaqFilter {
    compiled: Filter<Native<Val>>,
}

impl TryFrom<String> for JaqFilter {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for JaqFilter {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();

        let program = File {
            code: value,
            path: (),
        };

        // parse the filter
        let modules = loader
            .load(&arena, program)
            .map_err(|_e| Error::Load(value.to_string()))?;

        // compile the filter
        let compiled = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|_e| Error::Compile)?;

        Ok(Self { compiled })
    }
}

impl JaqFilter {
    fn run_internal(&self, input: Val) -> Result<Val, Error> {
        let inputs = RcIter::new(core::iter::empty());
        let ctx = Ctx::new([], &inputs);

        let mut i = self.compiled.run((ctx, input));
        let out = i.next().ok_or(Error::EmptyOutput)?;

        let out = out.map_err(|_e| Error::ExtractingOutput)?;

        // miffy needs to produce a single value, so we check if there are more
        if i.next().is_some() {
            Err(Error::MultipleOutputs)
        } else {
            Ok(out)
        }
    }

    pub fn run(&self, input: serde_json::Value) -> Result<serde_json::Value, Error> {
        let input = Val::from(input);
        let output = self.run_internal(input)?;
        Ok(output.into())
    }
}

#[cfg(test)]
mod test {
    use super::JaqFilter;
    use jaq_core::{Ctx, RcIter, load};
    use jaq_json::Val;
    use serde_json::json;

    #[test]
    fn test_jaq_abstraction() {
        let compiler = JaqFilter::try_from(".a").expect("should compile");
        let sample = Val::from(json!({"a": "wurst"}));

        let actual = compiler
            .run_internal(sample)
            .expect("should produce output");
        let expected = Val::from(json!("wurst"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_jaq_usage() {
        // not really a test, more a sandbox for JAQ usage, copied from https://docs.rs/jaq-core/2.1.1/jaq_core/index.html

        let input = json!(["Hello", "world"]);
        let program = File {
            // this filter produces multiple outputs which do not work for miffy
            code: ".[]",
            path: (),
        };

        use load::{Arena, File, Loader};

        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();

        // parse the filter
        let modules = loader.load(&arena, program).expect("do it");

        // compile the filter
        let filter = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .expect("do it");

        let inputs = RcIter::new(core::iter::empty());

        // iterator over the output values
        let mut out = filter.run((Ctx::new([], &inputs), Val::from(input)));

        assert_eq!(out.next(), Some(Ok(Val::from(json!("Hello")))));
        assert_eq!(out.next(), Some(Ok(Val::from(json!("world")))));
        assert_eq!(out.next(), None);
    }
}
