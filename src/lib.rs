//! Archetype is a lightweight snapshot testing library with builtin
//! diffing and test generation. It is largely used for golden testing
//! JSON output, but can be instrumented to verify other types of output
//! so long as the output goes to UTF-8 or raw bytes.

use serde::Serialize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::PathBuf;

/// Take a snapshot of a some UTF-8 encoded text under a file with the
/// name `key`.
///
/// If this is the first time the test is being run, write the snapshot.
/// In CI this will fail as we want to catch any snapshots not committed
/// to version control.
///
/// If the file does exist, compare the two line-by-line. Any
/// differences will be output to stdout. In future versions of this
/// function it may make more sense to write to a sink or produce a
/// buffer of text.
///
/// ```
/// archetype::snap_json("hello-world", &String::from("hello-world"));
/// ```
pub fn snap(key: &str, subject: String) {
    let path = {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("snapshots");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).ok();
        }
        dir.push(&format!("{}.snap", key));
        dir
    };
    if !path.exists() {
        if option_env!("CI").map(|v| v == "true").unwrap_or(false) {
            assert!(false, "snapshot missing for {}", key)
        };
        fs::write(path, subject).expect("should be able to write snapshot");
    } else {
        let stored = fs::read_to_string(&path).expect("should be able to read snapshot");
        let diff = TextDiff::from_lines(&stored, &subject);
        if diff.ratio() != 1.0 {
            println!("{}", format!(" ┏━━━━━━━━ {} ━━━━━", key));
            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-┃",
                    ChangeTag::Insert => "+┃",
                    ChangeTag::Equal => " ┃",
                };
                print!("{}{}", sign, change);
            }
            println!("{}", format!(" ┗━━━━━━━━ {} ━━━━━", key));
            assert!(false, "snapshot mismatch at {}", path.to_string_lossy());
        }
    }
}

/// Take a snapshot of JSON under a file with the name `key`.
///
/// If this is the first time the test is being run, write the snapshot.
/// In CI this will fail as we want to catch any snapshots not committed
/// to version control.
///
/// If the file does exist, compare the two line-by-line. Any
/// differences will be output to stdout. In future versions of this
/// function it may make more sense to write to a sink or produce a
/// buffer of text.
///
/// ```
/// archetype::snap_json("hello-world", &String::from("hello-world"));
/// ```
pub fn snap_json<A: Serialize>(key: &str, subject: &A) {
    snap(
        key,
        serde_json::to_string_pretty(subject).expect("should serialize"),
    );
}

/// Create a new test for the given fixture.
///
/// The fixture must be uniquely named and should take no arguments.
///
/// ```
/// use archetype;
///
/// mod json {
///     use serde::{Deserialize, Serialize};
///
///     #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
///     #[serde(rename_all = "kebab-case", tag = "type")]
///     pub enum Search {
///         ByTerm { term: String, },
///         ByIds { ids: Vec<i64> },
///     }
/// }
///
/// pub fn search_by_ids() -> json::Search {
///     json::Search::ByIds { ids: vec![4, 7, 9] }
/// }
///
/// pub fn search_by_term() -> json::Search {
///     json::Search::ByTerm { term: String::from("an example search term") }
/// }
///
/// archetype::snap_json_test!(search_by_term);
/// ```
#[macro_export]
macro_rules! snap_json_test {
    ($fixture:ident) => {
        paste::paste! {
            #[test]
            fn [<snapshot_$fixture>]() {
                crate::snap_json(std::stringify!($fixture), &$fixture());
            }
        }
    };
}

#[cfg(test)]
mod tests {

    // Some example JSON.
    mod json {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        #[serde(rename_all = "kebab-case", tag = "type")]
        pub enum Search {
            ByTerm { term: String },
            ByIds { ids: Vec<i64> },
        }
    }

    pub fn search_by_ids() -> json::Search {
        json::Search::ByIds { ids: vec![4, 7, 9] }
    }

    pub fn search_by_term() -> json::Search {
        json::Search::ByTerm {
            term: String::from("an example search term"),
        }
    }

    crate::snap_json_test!(search_by_term);
    crate::snap_json_test!(search_by_ids);
}
