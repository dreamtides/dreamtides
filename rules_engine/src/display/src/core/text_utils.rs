/// Removes `<color=...>...</color>` markup from a string, preserving inner
/// text.
///
/// Useful for button labels and other interface elements that need uncolored
/// output from strings that may contain color markup.
pub fn strip_colors(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;

    while let Some(start) = remaining.find("<color=") {
        result.push_str(&remaining[..start]);
        let after_tag = &remaining[start..];
        if let Some(close_bracket) = after_tag.find('>') {
            let after_open = &after_tag[close_bracket + 1..];
            if let Some(end_tag) = after_open.find("</color>") {
                result.push_str(&after_open[..end_tag]);
                remaining = &after_open[end_tag + "</color>".len()..];
            } else {
                result.push_str(after_open);
                return result;
            }
        } else {
            result.push_str(after_tag);
            return result;
        }
    }

    result.push_str(remaining);
    result
}
