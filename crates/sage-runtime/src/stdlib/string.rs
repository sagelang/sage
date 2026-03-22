//! String helper functions for the Sage standard library.

/// Find the index of a substring within a string (Unicode-aware).
/// Returns None if not found, Some(index) otherwise.
#[must_use]
pub fn str_index_of(haystack: &str, needle: &str) -> Option<i64> {
    haystack.find(needle).map(|byte_pos| {
        // Convert byte position to char position
        haystack[..byte_pos].chars().count() as i64
    })
}

/// Extract a substring by character indices (Unicode-aware).
/// Indices are inclusive start, exclusive end.
#[must_use]
pub fn str_slice(s: &str, start: i64, end: i64) -> String {
    let start = start.max(0) as usize;
    let end = end.max(0) as usize;
    s.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

/// Pad a string at the start to reach the target length (Unicode-aware).
#[must_use]
pub fn str_pad_start(s: &str, target_len: i64, pad: &str) -> String {
    let target_len = target_len.max(0) as usize;
    let current_len = s.chars().count();
    if current_len >= target_len || pad.is_empty() {
        return s.to_string();
    }
    let needed = target_len - current_len;
    let pad_chars: Vec<char> = pad.chars().collect();
    let mut result = String::new();
    for i in 0..needed {
        result.push(pad_chars[i % pad_chars.len()]);
    }
    result.push_str(s);
    result
}

/// Pad a string at the end to reach the target length (Unicode-aware).
#[must_use]
pub fn str_pad_end(s: &str, target_len: i64, pad: &str) -> String {
    let target_len = target_len.max(0) as usize;
    let current_len = s.chars().count();
    if current_len >= target_len || pad.is_empty() {
        return s.to_string();
    }
    let needed = target_len - current_len;
    let pad_chars: Vec<char> = pad.chars().collect();
    let mut result = s.to_string();
    for i in 0..needed {
        result.push(pad_chars[i % pad_chars.len()]);
    }
    result
}

/// Convert a Unicode code point to a single-character string.
/// Returns the Unicode replacement character for invalid code points.
#[must_use]
pub fn chr(code: i64) -> String {
    char::from_u32(code as u32)
        .unwrap_or('\u{FFFD}')
        .to_string()
}

/// Slice a list by indices (bounds-safe).
/// Indices are inclusive start, exclusive end.
#[must_use]
pub fn list_slice<T: Clone>(list: Vec<T>, start: i64, end: i64) -> Vec<T> {
    let len = list.len();
    let start = start.max(0) as usize;
    let end = end.max(0) as usize;
    let start = start.min(len);
    let end = end.min(len);
    if start >= end {
        return Vec::new();
    }
    list[start..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_slice() {
        assert_eq!(list_slice(vec![1, 2, 3, 4, 5], 1, 4), vec![2, 3, 4]);
        assert_eq!(list_slice(vec![1, 2, 3], 0, 10), vec![1, 2, 3]);
        assert_eq!(list_slice(vec![1, 2, 3], -5, 2), vec![1, 2]);
        assert_eq!(list_slice(vec![1, 2, 3], 5, 10), Vec::<i64>::new());
    }

    #[test]
    fn test_str_index_of() {
        assert_eq!(str_index_of("hello world", "world"), Some(6));
        assert_eq!(str_index_of("hello world", "foo"), None);
        assert_eq!(str_index_of("hello", ""), Some(0));
        // Unicode test
        assert_eq!(str_index_of("héllo wörld", "wörld"), Some(6));
    }

    #[test]
    fn test_str_slice() {
        assert_eq!(str_slice("hello", 1, 4), "ell");
        assert_eq!(str_slice("hello", 0, 5), "hello");
        assert_eq!(str_slice("hello", 3, 100), "lo");
        assert_eq!(str_slice("hello", -5, 3), "hel");
        // Unicode test
        assert_eq!(str_slice("héllo", 0, 3), "hél");
    }

    #[test]
    fn test_str_pad_start() {
        assert_eq!(str_pad_start("5", 3, "0"), "005");
        assert_eq!(str_pad_start("hello", 3, "x"), "hello");
        assert_eq!(str_pad_start("a", 5, "xy"), "xyxya");
    }

    #[test]
    fn test_str_pad_end() {
        assert_eq!(str_pad_end("5", 3, "0"), "500");
        assert_eq!(str_pad_end("hello", 3, "x"), "hello");
        assert_eq!(str_pad_end("a", 5, "xy"), "axyxy");
    }
}
