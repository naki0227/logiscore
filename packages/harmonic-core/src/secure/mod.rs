mod envelope;

pub use envelope::{open, seal, SecureEnvelope};

#[cfg(test)]
mod tests;
