use ahash::HashSet;

pub fn i64_safe_max() -> i64 {
    i64::MAX - 1
}

pub fn set_to_string<T: ToString>(set: &HashSet<T>, sep: &str) -> String {
    set.iter().map(|n|n.to_string()).collect::<Vec<String>>().join(sep)
}