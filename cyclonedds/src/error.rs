//! Module containing error types and utilities.

/// Result type specialized for DDS Errors.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can occur during DDS operations.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Error {
    /// Non specific error.
    NonSpecific,
    /// Feature unsupported.
    Unsupported,
    /// Bad parameter value.
    BadParameter,
    /// Precondition for operation not met.
    PreconditionNotMet,
    /// When an operation fails because of a lack of resources.
    OutOfResources,
    /// When a configurable feature is not enabled.
    NotEnabled,
    /// When an attempt is made to modify an immutable policy.
    ImmutablePolicy,
    /// When a policy is used with inconsistent values.
    InconsistentPolicy,
    /// When an attempt is made to delete something more than once.
    AlreadyDeleted,
    /// When a timeout has occurred.
    Timeout,
    /// When expected data is not provided.
    NoData,
    /// When a function is called when it should not be.
    IllegalOperation,
    /// When credentials are insufficient to use the function.
    NotAllowedBySecurity,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::NonSpecific => "non specific error occurred",
                Error::Unsupported => "unsupported feature",
                Error::BadParameter => "bad parameter provided",
                Error::PreconditionNotMet => "precondition for operation not met",
                Error::OutOfResources => "out of resources",
                Error::NotEnabled => "feature not enabled",
                Error::ImmutablePolicy => "attempted to modify immutable policy",
                Error::InconsistentPolicy => "policy has inconsistent values",
                Error::AlreadyDeleted => "entity already deleted",
                Error::Timeout => "timeout reached",
                Error::NoData => "data not provided",
                Error::IllegalOperation => "operation is illegal",
                Error::NotAllowedBySecurity => "insufficient credentials",
            }
        )
    }
}

impl std::error::Error for Error {}

/// Conversion trait for mapping status codes to the Result type.
pub(crate) trait IntoError<T>: Sized {
    type Error;

    fn into_error(self) -> Result<Self, Self::Error>;
}

impl IntoError<Error> for cyclonedds_sys::dds_return_t {
    type Error = Error;

    #[inline]
    fn into_error(self) -> Result<Self, Self::Error> {
        match self {
            result if result >= 0 => Ok(result),
            cyclonedds_sys::DDS_RETCODE_ERROR => Err(Self::Error::NonSpecific),
            cyclonedds_sys::DDS_RETCODE_UNSUPPORTED => Err(Self::Error::Unsupported),
            cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER => Err(Self::Error::BadParameter),
            cyclonedds_sys::DDS_RETCODE_PRECONDITION_NOT_MET => {
                Err(Self::Error::PreconditionNotMet)
            }
            cyclonedds_sys::DDS_RETCODE_OUT_OF_RESOURCES => Err(Self::Error::OutOfResources),
            cyclonedds_sys::DDS_RETCODE_NOT_ENABLED => Err(Self::Error::NotEnabled),
            cyclonedds_sys::DDS_RETCODE_IMMUTABLE_POLICY => Err(Self::Error::ImmutablePolicy),
            cyclonedds_sys::DDS_RETCODE_INCONSISTENT_POLICY => Err(Self::Error::InconsistentPolicy),
            cyclonedds_sys::DDS_RETCODE_ALREADY_DELETED => Err(Self::Error::AlreadyDeleted),
            cyclonedds_sys::DDS_RETCODE_TIMEOUT => Err(Self::Error::Timeout),
            cyclonedds_sys::DDS_RETCODE_NO_DATA => Err(Self::Error::NoData),
            cyclonedds_sys::DDS_RETCODE_ILLEGAL_OPERATION => Err(Self::Error::IllegalOperation),
            cyclonedds_sys::DDS_RETCODE_NOT_ALLOWED_BY_SECURITY => {
                Err(Self::Error::NotAllowedBySecurity)
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_is_success() {
        let actual = 0.into_error();
        let expected = Ok(0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_positive_is_success() {
        let actual = cyclonedds_sys::dds_return_t::MAX.into_error();
        let expected = Ok(i32::MAX);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_error_mapping() {
        let actual = cyclonedds_sys::DDS_RETCODE_ERROR.into_error();
        let expected = Err(Error::NonSpecific);
        assert_eq!(actual, expected);
        assert_eq!(
            format!("{}", actual.unwrap_err()),
            "non specific error occurred"
        );
        let actual = cyclonedds_sys::DDS_RETCODE_UNSUPPORTED.into_error();
        let expected = Err(Error::Unsupported);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "unsupported feature");
        let actual = cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER.into_error();
        let expected = Err(Error::BadParameter);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "bad parameter provided");
        let actual = cyclonedds_sys::DDS_RETCODE_PRECONDITION_NOT_MET.into_error();
        let expected = Err(Error::PreconditionNotMet);
        assert_eq!(actual, expected);
        assert_eq!(
            format!("{}", actual.unwrap_err()),
            "precondition for operation not met"
        );
        let actual = cyclonedds_sys::DDS_RETCODE_OUT_OF_RESOURCES.into_error();
        let expected = Err(Error::OutOfResources);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "out of resources");
        let actual = cyclonedds_sys::DDS_RETCODE_NOT_ENABLED.into_error();
        let expected = Err(Error::NotEnabled);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "feature not enabled");
        let actual = cyclonedds_sys::DDS_RETCODE_IMMUTABLE_POLICY.into_error();
        let expected = Err(Error::ImmutablePolicy);
        assert_eq!(actual, expected);
        assert_eq!(
            format!("{}", actual.unwrap_err()),
            "attempted to modify immutable policy"
        );
        let actual = cyclonedds_sys::DDS_RETCODE_INCONSISTENT_POLICY.into_error();
        let expected = Err(Error::InconsistentPolicy);
        assert_eq!(actual, expected);
        assert_eq!(
            format!("{}", actual.unwrap_err()),
            "policy has inconsistent values"
        );
        let actual = cyclonedds_sys::DDS_RETCODE_ALREADY_DELETED.into_error();
        let expected = Err(Error::AlreadyDeleted);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "entity already deleted");
        let actual = cyclonedds_sys::DDS_RETCODE_TIMEOUT.into_error();
        let expected = Err(Error::Timeout);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "timeout reached");
        let actual = cyclonedds_sys::DDS_RETCODE_NO_DATA.into_error();
        let expected = Err(Error::NoData);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "data not provided");
        let actual = cyclonedds_sys::DDS_RETCODE_ILLEGAL_OPERATION.into_error();
        let expected = Err(Error::IllegalOperation);
        assert_eq!(actual, expected);
        assert_eq!(format!("{}", actual.unwrap_err()), "operation is illegal");
        let actual = cyclonedds_sys::DDS_RETCODE_NOT_ALLOWED_BY_SECURITY.into_error();
        let expected = Err(Error::NotAllowedBySecurity);
        assert_eq!(actual, expected);
        assert_eq!(
            format!("{}", actual.unwrap_err()),
            "insufficient credentials"
        );
    }

    #[test]
    #[should_panic = "internal error: entered unreachable code"]
    fn test_out_of_bounds_error_panics() {
        let _ = cyclonedds_sys::dds_return_t::MIN.into_error();
    }
}
