# NTP Timestamp

NTP 64-bit timestamp[^1] implementation.

## NTP Timestamp format

```text
     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                            Seconds                            |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |                            Fraction                           |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### Timestamp field format

#### Seconds

Specifies the integer portion of the number of seconds since the epoch.

Size:

- 32 bits

Units:

- seconds

#### Fraction

Specifies the fractional portion of the number of seconds since the epoch.

Size:

- 32 bits.

Units:

- The unit is 2^32 seconds, which is roughly equal to 233 picoseconds.

Epoch:

- The epoch is 1 January 1900 at 00:00 UTC

Leap seconds:

- This timestamp format is affected by leap seconds (*this library doesn't automatically account for leap seconds*)

Resolution:

- The resolution is 2^32 seconds

Wraparound:

- This time format wraps around every 232 seconds, which is roughly 136 years. The next wraparound will occur in the year 2036.

## Install

Not published

## Examples

```rust
use ntp_timestamp::{DurationExt, NTPTimestamp};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

struct Msg {
    x: u8, 
    ts: u64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.ntp_from_unix().timestamp();
    let msg = Msg {
      x: 1, 
      ts
    };

    println!("{:?}", ts);

    Ok(())
}
```

## Project Status

No plans to release, please fork it.

## License

- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

## References

[RFC5905](https://www.rfc-editor.org/info/rfc5905)
[RFC8877](https://www.rfc-editor.org/rfc/rfc8877)

## Date

 `May 2024`

[^1]: [RFC5905](https://www.rfc-editor.org/info/rfc5905)
