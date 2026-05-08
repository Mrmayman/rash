use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct RashError<T> {
    pub trace: Vec<String>,
    pub kind: T,
}

impl<T: Display> Display for RashError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rash error: {}", self.kind)?;
        for t in &self.trace {
            write!(f, "\n  at {}", t)?;
        }
        Ok(())
    }
}

pub trait Trace {
    fn trace(self, t: &str) -> Self;
}

impl<T, E> Trace for Result<T, RashError<E>> {
    fn trace(mut self, t: &str) -> Self {
        if let Err(err) = &mut self {
            err.trace.push(t.to_owned());
        }
        self
    }
}

pub trait ErrorConvert<E, T> {
    fn to(self, a: &str, b: &str) -> Result<T, RashError<E>>;
}
