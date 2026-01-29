use crate::error::CommonError;

/// Common Result type alias
pub type CommonResult<T> = Result<T, CommonError>;
