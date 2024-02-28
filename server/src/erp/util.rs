use std::sync::OnceLock;

use ahash::HashSet;
use regex::Regex;

static RE: OnceLock<Regex> = OnceLock::new();

pub fn get_search_where_condition(col: &str, query: &str) -> String {
    let re: &Regex = RE.get_or_init(|| Regex::new(r"[\s+\(\)\-\:\@（）]").unwrap());
    let mut tmp = [0u8; 4];
    let qs: Vec<&str> = query
        .trim()
        .split(|a: char| re.is_match(a.encode_utf8(&mut tmp)))
        .collect();
    let mut conditions = Vec::with_capacity(qs.len());
    for q in qs {
        conditions.push(format!("{col} LIKE '%{q}%'"));
    }
    conditions.join(" AND ").into()
}

pub fn get_sorter_str(sorter: &str) -> &'static str {
    if sorter.contains(":descend") {
        "DESC"
    } else {
        "ASC"
    }
}

pub fn get_sort_col_str(col: &str) -> String {
    col.replace(":ascend", "").replace(":descend", "").into()
}

pub fn eq_or_not(reverse: Option<&HashSet<String>>, col: &str) -> &'static str {
    match reverse {
        Some(reverse) => {
            if reverse.contains(col) {
                "<>"
            } else {
                "="
            }
        }
        None => "=",
    }
}

pub fn in_or_not(reverse: Option<&HashSet<String>>, col: &str) -> &'static str {
    match reverse {
        Some(reverse) => {
            if reverse.contains(col) {
                " NOT IN "
            } else {
                " IN "
            }
        }
        None => " IN ",
    }
}

pub fn like_or_not(reverse: Option<&HashSet<String>>, col: &str) -> &'static str {
    match reverse {
        Some(reverse) => {
            if reverse.contains(col) {
                " NOT LIKE "
            } else {
                " LIKE "
            }
        }
        None => " LIKE ",
    }
}

