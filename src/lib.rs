#![warn(clippy::pedantic)]
#![warn(missing_docs)]
// #![warn(clippy::cargo)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(not(feature = "std"))]
use core::time;

#[cfg(feature = "std")]
use std::time;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// This is the number of seconds between the NTP epoch *1st January 1900* and
/// the Unix epoch *1st January 1970*.
pub const NTP_EPOCH_DELTA: time::Duration = time::Duration::from_secs(2_208_988_800);

const FRACTION_BITMASK: u64 = 0x0000_0000_FFFF_FFFF;
const SECONDS_BITMASK: u64 = 0xFFFF_FFFF_0000_0000;

const SEC_AS_MS: u64 = 1_000;
const SEC_AS_US: u32 = 1_000_000;
const SEC_AS_NS: u64 = 1_000_000_000;
const SEC_AS_PS: u64 = 1_000_000_000_000;

/// A struct representing an NTP timestamp.
#[must_use]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NTPTimestamp {
    seconds: u32,
    fraction: u32,
}

impl NTPTimestamp {
    /// Creates a new [`NTPTimestamp`] from seconds and fraction of seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let t = NTPTimestamp::new(1_000_000,0x4000_0000);
    /// ```
    pub fn new(seconds: u32, fraction: u32) -> Self {
        Self { seconds, fraction }
    }

    /// Returns [`NTPTimestamp`] as u64 ntp timestamp.
    ///
    /// # Examples
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let t = NTPTimestamp::new(1_000_000,0x4000_0000);
    /// let timestamp = t.timestamp();
    ///
    /// assert_eq!(timestamp, 4_294_968_369_741_824);
    /// ```
    #[must_use]
    pub fn timestamp(&self) -> u64 {
        Self::encode_to_u64(self.seconds, self.fraction)
    }

    /// Creates a new [`NTPTimestamp`] from a u64 ntp timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let timestamp = 4_294_968_369_741_824;
    /// let t = NTPTimestamp::from_ntp_timestamp(timestamp);
    ///
    /// assert_eq!(t.seconds(), 1_000_000);
    /// assert_eq!(t.fraction(),0x4000_0000);
    /// ```
    pub fn from_ntp_timestamp(ts: u64) -> Self {
        Self::decode_from_u64(ts)
    }

    /// Converts unix timestamp to [`NTPTimestamp`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let ts = 1640995200;
    /// let t = NTPTimestamp::from_unix_timestamp(ts);
    ///
    ///
    /// assert_eq!(t.seconds(), 3849984000);
    /// ```
    pub fn from_unix_timestamp(ts: u64) -> Self {
        let seconds = Self::from_unix_sec(ts);
        Self::new(seconds, 0)
    }

    /// Converts `NTPTimestamp` to Unix timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let t = NTPTimestamp::new(3849984000, 0);
    /// let ts = t.to_unix_timestamp();
    ///
    /// assert_eq!(ts, 1640995200);
    /// ```
    #[allow(clippy::cast_lossless)]
    #[must_use]
    pub fn to_unix_timestamp(&self) -> u64 {
        let seconds = u64::from(self.seconds()) - NTP_EPOCH_DELTA.as_secs();
        let fraction = u64::from(self.fraction) / u32::MAX as u64;

        seconds + fraction
    }

    /// Converts [`time::Duration`] to [`NTPTimestamp`].
    /// Expects a `Duration` since Unix epoch.
    pub fn from_unix_duration(duration: &time::Duration) -> Self {
        let seconds = Self::from_unix_sec(duration.as_secs());
        let fraction = Self::micros_fraction(duration.subsec_micros());

        Self::new(seconds, fraction)
    }

    /// Converts [`time::Duration`] to [`NTPTimestamp`].
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_duration(duration: &time::Duration) -> Self {
        let seconds = duration.as_secs() as u32;
        let fraction = Self::micros_fraction(duration.subsec_micros());

        Self::new(seconds, fraction)
    }

    /// Converts [`NTPTimestamp`] to [`time::Duration`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let t = NTPTimestamp::new(1_000_000, 0x4000_0000);
    /// let duration = t.to_duration();
    ///
    /// assert_eq!(duration.as_secs(), 1_000_000);
    /// assert_eq!(duration.subsec_micros(), 250_000);
    /// ```
    #[must_use]
    pub fn to_duration(&self) -> time::Duration {
        let secs = time::Duration::from_secs(u64::from(self.seconds));
        let nanos = self.fraction_as_ns();
        let fraction = time::Duration::from_nanos(nanos);

        secs + fraction
    }

    /// Returns [`NTPTimestamp`] as nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let t = NTPTimestamp::new(1, 0);
    /// let nanos = t.as_nanoseconds();
    ///
    /// assert_eq!(nanos, 1_000_000_000);
    /// ```
    #[must_use]
    pub fn as_nanoseconds(&self) -> u64 {
        u64::from(self.seconds) * SEC_AS_NS + self.fraction_as_ns()
    }

    /// Returns the seconds.
    #[must_use]
    pub fn seconds(&self) -> u32 {
        self.seconds
    }

    /// Returns the fraction of seconds.
    #[must_use]
    pub fn fraction(&self) -> u32 {
        self.fraction
    }

    /// Return fraction as milliseconds.
    #[must_use]
    pub fn fraction_as_ms(&self) -> u64 {
        (u64::from(self.fraction) * SEC_AS_MS) >> 32
    }

    /// Return fraction as microseconds.
    #[must_use]
    pub fn fraction_as_us(&self) -> u64 {
        (u64::from(self.fraction) * u64::from(SEC_AS_US)) >> 32
    }

    /// Return fraction as nanoseconds.
    #[must_use]
    pub fn fraction_as_ns(&self) -> u64 {
        (u64::from(self.fraction) * SEC_AS_NS) >> 32
    }

    /// Return fraction as picoseconds.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn fraction_as_ps(&self) -> u64 {
        ((u128::from(self.fraction) * u128::from(SEC_AS_PS)) >> 32) as u64
    }

    #[allow(clippy::cast_possible_truncation)]
    fn from_unix_sec(ts: u64) -> u32 {
        (ts + NTP_EPOCH_DELTA.as_secs()) as u32
    }

    #[allow(clippy::cast_possible_truncation)]
    fn micros_fraction(ts: u32) -> u32 {
        let ts = u64::from(ts);
        let us = u64::from(SEC_AS_US);
        let scale = u64::from(u32::MAX);

        ((ts * scale) / us) as u32
    }

    fn decode_from_u64(ts: u64) -> Self {
        let seconds = ((ts & SECONDS_BITMASK) >> 32) as u32;
        let fraction = (ts & FRACTION_BITMASK) as u32;

        Self::new(seconds, fraction)
    }

    fn encode_to_u64(high: u32, low: u32) -> u64 {
        (u64::from(high) << 32) | u64::from(low)
    }
}

#[cfg(feature = "std")]
impl NTPTimestamp {
    /// Returns the current system time [`NTPTimestamp`].
    ///
    /// # Panics
    ///
    /// This function panics if the system time is earlier than the UNIX epoch.
    ///
    /// # Examples
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let now = NTPTimestamp::now();
    /// ```
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    pub fn now() -> Self {
        let ts = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("System time is earlier than UNIX epoch");

        let seconds = Self::from_unix_sec(ts.as_secs());
        let fraction = Self::micros_fraction(ts.subsec_micros());

        Self::new(seconds, fraction)
    }

    /// Returns the NTP epoch as a [`std::time::SystemTime`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::NTPTimestamp;
    ///
    /// let epoch = NTPTimestamp::ntp_epoch();
    /// ```
    #[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
    #[must_use]
    pub fn ntp_epoch() -> time::SystemTime {
        time::UNIX_EPOCH + NTP_EPOCH_DELTA
    }
}

/// [`Default::default`] returns the current system time [`NTPTimestamp`].
///
/// # Examples
///
/// ```
/// use ntp_timestamp::NTPTimestamp;
///
/// let now = NTPTimestamp::default();
/// ```
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl Default for NTPTimestamp {
    fn default() -> Self {
        Self::now()
    }
}

/// Serialize the [`NTPTimestamp`] as a u64 `NTP` timestamp.
///
/// # Examples
///
/// ```
/// use ntp_timestamp::NTPTimestamp;
/// use serde_json::json;
///
/// let t = NTPTimestamp::new(1_000_000, 0x4000_0000);
/// let serialized = serde_json::to_string(&t).unwrap();
///
/// assert_eq!(serialized, format!("{}", t.timestamp()));
/// ```
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
#[cfg(feature = "serde")]
impl Serialize for NTPTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.timestamp().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for NTPTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<NTPTimestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = u64::deserialize(deserializer)?;
        Ok(NTPTimestamp::from_ntp_timestamp(timestamp))
    }
}

/// Extends [`core::time::Duration`] with methods to convert to [`NTPTimestamp`].
#[cfg(not(feature = "std"))]
impl DurationExt for time::Duration {
    /// Returns the [`NTPTimestamp`] representation of the [`std::time::Duration`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::{DurationExt, NTPTimestamp};
    ///
    /// let duration = core::time::Duration::new(1_000_000, 250_000);
    /// let t = duration.ntp_timestamp();
    ///
    /// assert_eq!(t.seconds(), 1_000_000);
    /// assert_eq!(t.fraction(), 1073741);
    /// ```
    fn ntp_timestamp(&self) -> NTPTimestamp {
        NTPTimestamp::from_duration(self)
    }

    fn ntp_from_unix(&self) -> NTPTimestamp {
        NTPTimestamp::from_unix_duration(self)
    }
}

/// Extends [`std::time::Duration`] with methods to convert to [`NTPTimestamp`].
#[cfg(feature = "std")]
impl DurationExt for time::Duration {
    /// Returns the [`NTPTimestamp`] representation of the [`time::Duration`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ntp_timestamp::{DurationExt, NTPTimestamp};
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.ntp_from_unix();
    ///
    /// # println!("{:?}", ts);
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn ntp_from_unix(&self) -> NTPTimestamp {
        NTPTimestamp::from_unix_duration(self)
    }

    fn ntp_timestamp(&self) -> NTPTimestamp {
        NTPTimestamp::from_duration(self)
    }
}

/// Extends [`time::Duration`] with methods to convert to [`NTPTimestamp`].
pub trait DurationExt {
    /// Returns the [`NTPTimestamp`] representation of the `Unix` [`time::Duration`].
    fn ntp_from_unix(&self) -> NTPTimestamp;

    /// Returns the [`NTPTimestamp`] representation of the [`time::Duration`].
    fn ntp_timestamp(&self) -> NTPTimestamp;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let t = NTPTimestamp::new(1_000_000, 0x4000_0000);
        assert_eq!(t.seconds(), 1_000_000);
        assert_eq!(t.fraction(), 0x4000_0000);
    }

    #[test]
    fn test_timestamp() {
        let t = NTPTimestamp::new(1_000_000, 0x4000_0000);
        let timestamp = t.timestamp();
        assert_eq!(timestamp, 4_294_968_369_741_824);
    }

    #[test]
    fn test_from_ntp_timestamp() {
        let timestamp = 4_294_968_369_741_824;
        let t = NTPTimestamp::from_ntp_timestamp(timestamp);
        assert_eq!(t.seconds(), 1_000_000);
        assert_eq!(t.fraction(), 0x4000_0000);
    }

    #[test]
    fn test_to_duration() {
        let t = NTPTimestamp::new(1_000_000, 0x4000_0000);
        let duration = t.to_duration();

        assert_eq!(t.timestamp(), 4_294_968_369_741_824);
        assert_eq!(duration.as_secs(), 1_000_000);
        assert_eq!(duration.subsec_micros(), 250_000);
    }

    #[test]
    fn test_as_nanoseconds() {
        let t = NTPTimestamp::new(1, 0);
        let nanos = t.as_nanoseconds();
        assert_eq!(nanos, 1_000_000_000);
    }

    #[test]
    fn test_fraction_as_ms() {
        let t = NTPTimestamp::new(0, 0x4000_0000);
        let fraction_ms = t.fraction_as_ms();
        assert_eq!(fraction_ms, 250);
    }

    #[test]
    fn test_fraction_as_ns() {
        let t = NTPTimestamp::new(0, 0x4000_0000);
        let fraction_ns = t.fraction_as_ns();

        assert_eq!(fraction_ns, 250_000_000);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_ntp_epoch() {
        let epoch = NTPTimestamp::ntp_epoch();

        assert!(epoch.duration_since(time::UNIX_EPOCH).unwrap() >= NTP_EPOCH_DELTA);
    }

    #[test]
    fn test_fraction_convertions() {
        let t = NTPTimestamp::new(0, 0x2000_0000);

        assert_eq!(t.fraction_as_ms(), 125);
        assert_eq!(t.fraction_as_us(), 125_000);
        assert_eq!(t.fraction_as_ns(), 125_000_000);
        assert_eq!(t.fraction_as_ps(), 125_000_000_000);
    }
}
