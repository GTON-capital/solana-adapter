use std::error;




pub type WrappedResult<T> = Result<T, Box<dyn error::Error>>;