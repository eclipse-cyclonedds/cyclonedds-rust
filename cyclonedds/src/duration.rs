/// A relative span of time represented as nanoseconds.
///
/// Used in DDS for timeouts, lease durations, deadlines, and other
/// interval-based [`QoS`](crate::QoS) policies.
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct Duration {
    pub(crate) inner: cyclonedds_sys::dds_duration_t,
}

impl Duration {
    /// A sentinel value representing an infinite timeout.
    ///
    /// Pass this to any API that accepts a [`Duration`] to block indefinitely.
    pub const INFINITE: Self = Duration {
        inner: cyclonedds_sys::DURATION_INFINITE,
    };

    /// Creates a [`Duration`] from a nanosecond value.
    ///
    /// Unlike [`std::time::Duration::from_nanos`], this accepts an [`i64`]
    /// rather than a [`u64`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// let d = Duration::from_nanos(1_000_000);
    /// ```
    #[must_use]
    pub const fn from_nanos(nanos: i64) -> Self {
        Self { inner: nanos }
    }

    /// Creates a [`Duration`] from a millisecond value.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// let d = Duration::from_millis(100);
    /// ```
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self {
            inner: millis * 1_000_000,
        }
    }

    /// Creates a [`Duration`] from a second value.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// let d = Duration::from_secs(5);
    /// ```
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self {
            inner: secs * 1_000_000_000,
        }
    }

    /// Returns the duration in nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1).as_nanos(), 1_000_000_000);
    /// ```
    #[must_use]
    pub const fn as_nanos(&self) -> i64 {
        self.inner
    }

    /// Returns the duration in whole milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// assert_eq!(Duration::from_secs(1).as_millis(), 1_000);
    /// ```
    #[must_use]
    pub const fn as_millis(&self) -> i64 {
        self.inner / 1_000_000
    }

    /// Returns the duration in whole seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// assert_eq!(Duration::from_millis(1_500).as_secs(), 1);
    /// ```
    #[must_use]
    pub const fn as_secs(&self) -> i64 {
        self.inner / 1_000_000_000
    }

    /// Returns `true` if this duration equals [`Duration::INFINITE`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// assert!(Duration::INFINITE.is_infinite());
    /// assert!(!Duration::from_secs(5).is_infinite());
    /// ```
    #[must_use]
    pub const fn is_infinite(&self) -> bool {
        self.inner == Self::INFINITE.inner
    }

    /// Adds two durations, returning `None` on overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// let d = Duration::from_secs(1).checked_add(Duration::from_secs(2));
    /// assert_eq!(d, Some(Duration::from_secs(3)));
    /// ```
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.inner
            .checked_add(rhs.inner)
            .map(|inner| Self { inner })
    }

    /// Subtracts a duration, returning `None` on underflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// let d = Duration::from_secs(3).checked_sub(Duration::from_secs(1));
    /// assert_eq!(d, Some(Duration::from_secs(2)));
    /// ```
    #[must_use]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.inner
            .checked_sub(rhs.inner)
            .map(|inner| Self { inner })
    }

    /// Multiplies a duration by a scalar, returning `None` on overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Duration;
    ///
    /// let d = Duration::from_secs(2).checked_mul(3);
    /// assert_eq!(d, Some(Duration::from_secs(6)));
    /// ```
    #[must_use]
    pub fn checked_mul(self, rhs: i64) -> Option<Self> {
        self.inner.checked_mul(rhs).map(|inner| Self { inner })
    }
}

impl std::ops::Add for Duration {
    type Output = Self;

    /// Adds two durations.
    ///
    /// # Panics
    ///
    /// Panics on overflow. Use [`checked_add`](Duration::checked_add) for a
    /// non-panicking alternative.
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding durations")
    }
}

impl std::ops::Sub for Duration {
    type Output = Self;

    /// Subtracts a duration.
    ///
    /// # Panics
    ///
    /// Panics on underflow. Use [`checked_sub`](Duration::checked_sub) for a
    /// non-panicking alternative.
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
            .expect("underflow when subtracting durations")
    }
}

impl std::ops::Mul<i64> for Duration {
    type Output = Self;

    /// Multiplies a duration by a scalar.
    ///
    /// # Panics
    ///
    /// Panics on overflow. Use [`checked_mul`](Duration::checked_mul) for a
    /// non-panicking alternative.
    fn mul(self, rhs: i64) -> Self::Output {
        self.checked_mul(rhs)
            .expect("overflow when multiplying duration")
    }
}

impl TryFrom<std::time::Duration> for Duration {
    type Error = crate::Error;

    fn try_from(value: std::time::Duration) -> Result<Self, Self::Error> {
        let inner = cyclonedds_sys::dds_duration_t::try_from(value.as_nanos())
            .map_err(|_err| crate::Error::BadParameter)?;

        if inner == Self::INFINITE.inner {
            Err(crate::Error::BadParameter)
        } else {
            Ok(Self { inner })
        }
    }
}

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        std::time::Duration::from_nanos(duration.inner.cast_unsigned())
    }
}

impl std::fmt::Debug for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Duration::INFINITE => f.write_str("Duration::INFINITE"),
            duration => f
                .debug_tuple("Duration")
                .field(&format_args!("{}ns", duration.inner))
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_from_nanos() {
        let nanos = 1_000_000_000;
        let duration = Duration::from_nanos(nanos);
        assert_eq!(duration.inner, nanos);
    }
    #[test]
    fn test_duration_from_millis() {
        let duration = Duration::from_millis(100);
        assert_eq!(duration.inner, 100_000_000);
    }

    #[test]
    fn test_duration_from_secs() {
        let duration = Duration::from_secs(5);
        assert_eq!(duration.inner, 5_000_000_000);
    }

    #[test]
    fn test_duration_as_nanos() {
        let duration = Duration::from_nanos(1_000_000_000);
        assert_eq!(duration.as_nanos(), 1_000_000_000);
    }

    #[test]
    fn test_duration_as_millis() {
        let duration = Duration::from_secs(1);
        assert_eq!(duration.as_millis(), 1_000);
    }

    #[test]
    fn test_duration_from_std_duration() {
        let nanos = 1_000_000_000;
        let standard = std::time::Duration::from_nanos(nanos as u64);
        let duration = Duration::try_from(standard).unwrap();
        assert_eq!(duration.inner, nanos);
    }

    #[test]
    fn test_duration_from_infinite_std_duration() {
        let nanos = Duration::INFINITE.inner;
        let standard = std::time::Duration::from_nanos(nanos as u64);
        let result = Duration::try_from(standard).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_duration_from_out_of_range_std_duration() {
        let nanos = u64::MAX;
        let standard = std::time::Duration::from_nanos(nanos);
        let result = Duration::try_from(standard).unwrap_err();
        assert_eq!(result, crate::Error::BadParameter);
    }

    #[test]
    fn test_duration_as_millis_truncates() {
        let duration = Duration::from_millis(1) + Duration::from_nanos(999_999);
        assert_eq!(duration.as_millis(), 1);
    }

    #[test]
    fn test_duration_as_secs() {
        let duration = Duration::from_millis(1_500);
        assert_eq!(duration.as_secs(), 1);
    }

    #[test]
    fn test_duration_as_secs_truncates() {
        let duration = Duration::from_millis(999);
        assert_eq!(duration.as_secs(), 0);
    }

    #[test]
    fn test_duration_is_infinite() {
        assert!(Duration::INFINITE.is_infinite());
    }

    #[test]
    fn test_duration_is_not_infinite() {
        assert!(!Duration::from_secs(5).is_infinite());
    }

    #[test]
    fn test_duration_checked_add() {
        let result = Duration::from_secs(1).checked_add(Duration::from_secs(2));
        assert_eq!(result, Some(Duration::from_secs(3)));
    }

    #[test]
    fn test_duration_checked_add_overflow() {
        let result = Duration::from_nanos(i64::MAX).checked_add(Duration::from_nanos(1));
        assert_eq!(result, None);
    }

    #[test]
    fn test_duration_add() {
        let result = Duration::from_secs(1) + Duration::from_secs(2);
        assert_eq!(result, Duration::from_secs(3));
    }

    #[test]
    #[should_panic(expected = "overflow when adding durations")]
    fn test_duration_add_overflow() {
        let _ = Duration::from_nanos(i64::MAX) + Duration::from_nanos(1);
    }
    #[test]
    fn test_duration_checked_sub() {
        let result = Duration::from_secs(3).checked_sub(Duration::from_secs(1));
        assert_eq!(result, Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_duration_checked_sub_underflow() {
        let result = Duration::from_nanos(i64::MIN).checked_sub(Duration::from_nanos(1));
        assert_eq!(result, None);
    }

    #[test]
    fn test_duration_sub() {
        let result = Duration::from_secs(3) - Duration::from_secs(1);
        assert_eq!(result, Duration::from_secs(2));
    }

    #[test]
    #[should_panic(expected = "underflow when subtracting durations")]
    fn test_duration_sub_underflow() {
        let _ = Duration::from_nanos(i64::MIN) - Duration::from_nanos(1);
    }

    #[test]
    fn test_duration_checked_mul() {
        let result = Duration::from_secs(2).checked_mul(3);
        assert_eq!(result, Some(Duration::from_secs(6)));
    }

    #[test]
    fn test_duration_checked_mul_overflow() {
        let result = Duration::from_nanos(i64::MAX).checked_mul(2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_duration_mul() {
        let result = Duration::from_secs(2) * 3;
        assert_eq!(result, Duration::from_secs(6));
    }

    #[test]
    fn test_to_std_duration() {
        let nanos = 11_222_33;
        let duration = Duration::from_nanos(nanos);
        let expected = std::time::Duration::from_nanos(nanos as u64);
        let actual = std::time::Duration::from(duration);
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overflow when multiplying duration")]
    fn test_duration_mul_overflow() {
        let _ = Duration::from_nanos(i64::MAX) * 2;
    }

    #[test]
    fn test_duration_debug() {
        assert_eq!(format!("{:?}", Duration::default()), "Duration(0ns)");
        assert_eq!(format!("{:?}", Duration::INFINITE), "Duration::INFINITE");
        assert_eq!(format!("{:?}", Duration::from_nanos(42)), "Duration(42ns)");
    }
}
