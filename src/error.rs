pub enum Error {}
/// An alias for a `Result` with [`databento::Error`](crate::Error) as the error type.
pub type Result<T> = std::result::Result<T, Error>;
