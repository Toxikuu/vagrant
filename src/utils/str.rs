/// Returns the basename of a string, ie everything after the final slash, or just the string if
/// there are no slashes
#[inline]
pub fn basename(s: &str) -> &str {
    s.rsplit_once('/').map_or(s, |s| s.1)
}
