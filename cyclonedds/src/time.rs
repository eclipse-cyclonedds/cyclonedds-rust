use crate::Duration;

/// An absolute point in time represented as nanoseconds since the UNIX epoch.
///
/// Used in DDS for sample source timestamps and other time-stamped metadata.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Time {
    pub(crate) inner: cyclonedds_sys::dds_time_t,
}

impl Time {
    /// A sentinel value representing a time that will never occur.
    ///
    /// Used in DDS APIs to indicate an invalid or unset timestamp.
    pub const NEVER: Self = Time {
        inner: cyclonedds_sys::TIME_NEVER,
    };

    /// Creates a [`Time`] from a nanosecond timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Time;
    ///
    /// let t = Time::from_nanos(1_000_000_000);
    /// ```
    #[must_use]
    pub const fn from_nanos(nanos: i64) -> Self {
        Self { inner: nanos }
    }

    /// Creates a [`Time`] from a millisecond timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Time;
    ///
    /// let t = Time::from_millis(1_000);
    /// ```
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self {
            inner: millis * 1_000_000,
        }
    }

    /// Creates a [`Time`] from a second timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Time;
    ///
    /// let t = Time::from_secs(1);
    /// ```
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self {
            inner: secs * 1_000_000_000,
        }
    }

    /// Returns the timestamp as nanoseconds since the UNIX epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Time;
    ///
    /// let t = Time::from_secs(1);
    /// assert_eq!(t.as_nanos(), 1_000_000_000);
    /// ```
    #[must_use]
    pub const fn as_nanos(&self) -> i64 {
        self.inner
    }

    /// Returns the time elapsed since this timestamp as a [`Duration`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::BadParameter`](crate::Error::BadParameter) if the
    /// system time cannot be converted or if this timestamp is in the future.
    pub fn elapsed(&self) -> crate::Result<Duration> {
        Time::try_from(std::time::SystemTime::now()).and_then(|now| {
            if self.inner <= now.inner {
                let nanos = now.inner - self.inner;
                Ok(Duration::from_nanos(nanos))
            } else {
                Err(crate::Error::BadParameter)
            }
        })
    }

    /// Adds a [`Duration`] to this time, returning `None` on overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Duration, Time};
    ///
    /// let t = Time::from_secs(1).checked_add(Duration::from_secs(1));
    /// assert_eq!(t, Some(Time::from_secs(2)));
    /// ```
    #[must_use]
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.inner
            .checked_add(duration.inner)
            .map(|inner| Self { inner })
    }

    /// Subtracts a [`Duration`] from this time, returning `None` on underflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::{Duration, Time};
    ///
    /// let t = Time::from_secs(2).checked_sub(Duration::from_secs(1));
    /// assert_eq!(t, Some(Time::from_secs(1)));
    /// ```
    #[must_use]
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.inner
            .checked_sub(duration.inner)
            .map(|inner| Self { inner })
    }
}

impl TryFrom<std::time::SystemTime> for Time {
    type Error = crate::Error;

    fn try_from(value: std::time::SystemTime) -> Result<Self, Self::Error> {
        let value = value
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(|_err| crate::Error::BadParameter)?;

        let inner = cyclonedds_sys::dds_time_t::try_from(value.as_nanos())
            .map_err(|_err| crate::Error::BadParameter)?;

        if inner == Self::NEVER.inner {
            Err(crate::Error::BadParameter)
        } else {
            Ok(Self { inner })
        }
    }
}

impl From<Time> for std::time::SystemTime {
    fn from(time: Time) -> Self {
        std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_nanos(time.inner.cast_unsigned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Duration;

    #[test]
    fn test_time_create() {
        let nanos = 1_000_000_000;
        let time = Time::from_nanos(nanos);
        assert_eq!(time.inner, nanos);
    }

    #[test]
    fn test_time_from_std_system_time() {
        let nanos = 1_000_000_000;
        let standard =
            std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(nanos as u64);
        let time = Time::try_from(standard).unwrap();
        assert_eq!(time.inner, nanos);
    }

    #[test]
    // Windows SystemTime uses 100ns intervals in FILETIME, so Time::NEVER is
    // lost in the round-trip (it rounds down by 7ns).
    #[cfg(not(target_os = "windows"))]
    fn test_time_from_never_std_system_time() {
        let nanos = Time::NEVER.inner as u64;
        let standard = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(nanos);
        let result = Time::try_from(standard).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_time_from_out_of_range_std_system_time() {
        let standard = std::time::SystemTime::UNIX_EPOCH - std::time::Duration::from_nanos(100);
        let result = Time::try_from(standard).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);

        let nanos = u64::MAX;
        let standard = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(nanos);
        let result = Time::try_from(standard).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_time_from_millis() {
        let time = Time::from_millis(1_000);
        assert_eq!(time.inner, 1_000_000_000);
    }

    #[test]
    fn test_time_from_secs() {
        let time = Time::from_secs(1);
        assert_eq!(time.inner, 1_000_000_000);
    }

    #[test]
    fn test_time_as_nanos() {
        let time = Time::from_nanos(1_000_000_000);
        assert_eq!(time.as_nanos(), 1_000_000_000);
    }

    #[test]
    fn test_time_checked_add() {
        let result = Time::from_secs(1).checked_add(Duration::from_secs(2));
        assert_eq!(result, Some(Time::from_secs(3)));
    }

    #[test]
    fn test_time_checked_add_overflow() {
        let result = Time::from_nanos(i64::MAX).checked_add(Duration::from_nanos(1));
        assert_eq!(result, None);
    }

    #[test]
    fn test_time_checked_sub() {
        let result = Time::from_secs(3).checked_sub(Duration::from_secs(1));
        assert_eq!(result, Some(Time::from_secs(2)));
    }

    #[test]
    fn test_time_checked_sub_underflow() {
        let result = Time::from_nanos(i64::MIN).checked_sub(Duration::from_nanos(1));
        assert_eq!(result, None);
    }

    #[test]
    fn test_time_elapsed_increases() {
        let before = Time::try_from(std::time::SystemTime::now()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = before.elapsed().unwrap();
        assert!(elapsed.as_nanos() >= 10_000_000);
    }

    #[test]
    fn test_time_elapsed_future_returns_error() {
        let future = Time::from_nanos(i64::MAX);
        assert_eq!(future.elapsed().unwrap_err(), crate::Error::BadParameter);
    }

    #[test]
    fn test_to_std_system_time() {
        let nanos = 11_222_33;
        let time = Time::from_nanos(nanos);
        let expected =
            std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(nanos as u64);
        let actual: std::time::SystemTime = time.into();
        assert_eq!(actual, expected);
    }
}
