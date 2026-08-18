#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRange;

impl ByteRange {
    #[must_use]
    pub const fn length(self) -> u64 {
        self.end - self.start + 1
    }
}

pub fn parse_single_range(value: &str, size: u64) -> Result<ByteRange, InvalidRange> {
    let value = value.strip_prefix("bytes=").ok_or(InvalidRange)?;
    if value.contains(',') || size == 0 {
        return Err(InvalidRange);
    }
    let (start, end) = value.split_once('-').ok_or(InvalidRange)?;

    if start.is_empty() {
        let suffix = end.parse::<u64>().map_err(|_| InvalidRange)?;
        if suffix == 0 {
            return Err(InvalidRange);
        }
        let length = suffix.min(size);
        return Ok(ByteRange {
            start: size - length,
            end: size - 1,
        });
    }

    let start = start.parse::<u64>().map_err(|_| InvalidRange)?;
    if start >= size {
        return Err(InvalidRange);
    }
    let end = if end.is_empty() {
        size - 1
    } else {
        end.parse::<u64>().map_err(|_| InvalidRange)?.min(size - 1)
    };
    if end < start {
        return Err(InvalidRange);
    }
    Ok(ByteRange { start, end })
}

#[cfg(test)]
mod tests {
    use super::{parse_single_range, ByteRange};

    #[test]
    fn parses_closed_open_and_suffix_ranges() {
        assert_eq!(
            parse_single_range("bytes=2-5", 10),
            Ok(ByteRange { start: 2, end: 5 })
        );
        assert_eq!(
            parse_single_range("bytes=7-", 10),
            Ok(ByteRange { start: 7, end: 9 })
        );
        assert_eq!(
            parse_single_range("bytes=-3", 10),
            Ok(ByteRange { start: 7, end: 9 })
        );
        assert_eq!(
            parse_single_range("bytes=-20", 10),
            Ok(ByteRange { start: 0, end: 9 })
        );
    }

    #[test]
    fn rejects_invalid_or_unsatisfiable_ranges() {
        for value in [
            "items=0-1",
            "bytes=",
            "bytes=10-",
            "bytes=5-2",
            "bytes=0-1,4-5",
            "bytes=-0",
        ] {
            assert!(parse_single_range(value, 10).is_err(), "{value}");
        }
        assert!(parse_single_range("bytes=0-0", 0).is_err());
    }
}
