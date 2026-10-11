//! Formatting: filling copy templates, dollars, and integer rounding. The page
//! draws these strings as they are.

/// Replaces each `{name}` in `template` with its value.
pub fn fill(template: &str, vars: &[(&str, String)]) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// Whole dollars with thousands separators: 4000 -> "$4,000".
pub fn money(dollars: u32) -> String {
    let digits = dollars.to_string();
    let mut out = String::from("$");
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A count of tenths as a decimal: 234 -> "23.4".
pub fn tenths(value: u32) -> String {
    format!("{}.{}", value / 10, value % 10)
}

/// `num / den`, rounded half up. `den` must be at least 1.
pub fn div_round(num: u64, den: u64) -> u64 {
    (2 * num + den) / (2 * den)
}

/// The smallest multiple of `step` that is at least `value`.
pub fn ceil_to(value: u32, step: u32) -> u32 {
    value.div_ceil(step) * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_groups_thousands() {
        assert_eq!(money(0), "$0");
        assert_eq!(money(77), "$77");
        assert_eq!(money(4000), "$4,000");
        assert_eq!(money(1234567), "$1,234,567");
    }

    #[test]
    fn fill_replaces_every_named_slot() {
        assert_eq!(
            fill("{a} and {b} and {a}", &[("a", "x".into()), ("b", "y".into())]),
            "x and y and x"
        );
    }

    #[test]
    fn rounding_helpers() {
        assert_eq!(tenths(234), "23.4");
        assert_eq!(tenths(5), "0.5");
        assert_eq!(div_round(5, 10), 1);
        assert_eq!(div_round(4, 10), 0);
        assert_eq!(ceil_to(81, 5), 85);
        assert_eq!(ceil_to(80, 5), 80);
    }
}
