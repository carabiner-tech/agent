use llm_diff::FileDiff;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use std::{io::Cursor, path::PathBuf};

#[test]
fn test_normal_diff() {
    let content = "foo\nbar\nbaz";
    let lines: Vec<String> = Cursor::new(content)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let diff_str = r#"
--- a.txt
+++ b.txt    
@@ -1,3 +1,3 @@
 foo
- bar
+ qux
 baz
"#
    .trim();
    let diff = FileDiff::parse(diff_str).unwrap();

    let applied = diff.apply(&lines).unwrap();
    let expected = vec!["foo", "qux", "baz"];
    assert_eq!(applied, expected);
}

#[test]
fn test_missing_file_headers() {
    let content = "foo\nbar\nbaz";
    let lines: Vec<String> = Cursor::new(content)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let diff_str = r#"
@@ -1,3 +1,3 @@
 foo
- bar
+ qux
 baz
"#
    .trim();
    let diff = FileDiff::parse(diff_str).unwrap();

    let applied = diff.apply(&lines).unwrap();
    let expected = vec!["foo", "qux", "baz"];
    assert_eq!(applied, expected);
}

#[test]
fn test_added_line_at_start() {
    let content = "foo\nbar\nbaz";
    let lines: Vec<String> = Cursor::new(content)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let diff_str = r#"
@@ -1,1 +1,1 @@
+ qux
"#
    .trim();
    let diff = FileDiff::parse(diff_str).unwrap();

    let applied = diff.apply(&lines).unwrap();
    let expected = vec!["qux", "foo", "bar", "baz"];
    assert_eq!(applied, expected);
}

#[test]
fn test_add_line_at_bottom() {
    let content = "foo\nbar\nbaz";
    let lines: Vec<String> = Cursor::new(content)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let diff_str = r#"
@@ -3,1 +3,1 @@
 foo
 bar
 baz
+ qux
"#
    .trim();
    let diff = FileDiff::parse(diff_str).unwrap();

    let applied = diff.apply(&lines).unwrap();
    let expected = vec!["foo", "bar", "baz", "qux"];
    assert_eq!(applied, expected);
}

#[test]
fn test_no_spaces_after_line_add() {
    let content = "foo\nbar\nbaz";
    let lines: Vec<String> = Cursor::new(content)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let diff_str = r#"
@@ -1,1 +1,1 @@
 foo
 bar
+qux
 baz
"#
    .trim();
    let diff = FileDiff::parse(diff_str).unwrap();

    let applied = diff.apply(&lines).unwrap();
    let expected = vec!["foo", "bar", "qux", "baz"];
    assert_eq!(applied, expected);
}
