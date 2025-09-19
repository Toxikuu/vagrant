// utils/shortform.rs

pub fn get_shortform(maybe_short: &str) -> String {
    if ! maybe_short.contains("github.com") {
        return maybe_short.to_string()
    }

    let parts = maybe_short.split('/').collect::<Vec<_>>();

    if parts.len() > 3 {
        parts[3..5].join("/").trim_end_matches(".git").to_string()
    } else {
        maybe_short.to_string()
    }
}

pub fn is_shortform(maybe_short: &str) -> bool {
    maybe_short.split('/').count() == 2
}

pub fn get_longform(maybe_short: &str) -> String {
    if is_shortform(maybe_short) {
        format!("https://github.com/{maybe_short}.git")
    } else {
        maybe_short.to_string()
    }
}
