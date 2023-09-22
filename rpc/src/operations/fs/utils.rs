use std::{error::Error, path::PathBuf};

pub(crate) async fn read_lines(
    path: PathBuf,
    start_line: usize,
    end_line: usize,
) -> Result<(Vec<String>, usize, usize), Box<dyn Error>> {
    // First check - is the path relative or in CWD?
    match path.is_relative() {
        true => {}
        false => {
            if !path.starts_with(std::env::current_dir()?) {
                return Err("Path must be a sub-directory of the current working directory".into());
            }
        }
    }

    // Second check - does file exist and can be read?
    let content = tokio::fs::read_to_string(path).await?;

    // Gotcha here: .lines() will strip trailing \n so foo\nbar\nbaz is the same as foo\nbar\nbaz\n
    let has_trailing_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    if has_trailing_newline {
        lines.push("".into());
    }

    // Third check, are start / end lines within index? Don't want operations to panic.
    // Note the incoming start / end values are 1-indexed because LLMs work better calling ops
    // that deal with line numbers when we let them start at 1, but we want to downcast to 0-indexed
    // here for index check and for the operations to use downstream.
    let start_line = start_line - 1;
    let end_line = end_line - 1;
    if start_line > end_line {
        return Err("End line is greater than Start line".into());
    }
    if start_line >= lines.len() {
        return Err("Start line is out of index".into());
    }
    if end_line >= lines.len() {
        return Err("End line is out of index".into());
    }
    Ok((lines, start_line, end_line))
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use tempfile::TempDir;

    use super::*;

    #[rstest::fixture]
    fn tmp_dir() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        dir
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_no_trailing_new_line(_tmp_dir: TempDir) {
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3",
        )
        .unwrap();
        let path = PathBuf::from("test.txt");
        let (lines, _start_line, _end_line) = read_lines(path, 1, 2).await.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines.last().unwrap(), "line3")
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_preserves_trailing_new_lines(_tmp_dir: TempDir) {
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3\n",
        )
        .unwrap();
        let path = PathBuf::from("test.txt");
        let (lines, _start_line, _end_line) = read_lines(path, 1, 2).await.unwrap();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[3], "");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_preserves_inline_whitespace(_tmp_dir: TempDir) {
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"  line1\nline2  \n  line3  ",
        )
        .unwrap();
        let path = PathBuf::from("test.txt");
        let (lines, _start_line, _end_line) = read_lines(path, 1, 2).await.unwrap();
        assert_eq!(lines[0], "  line1");
        assert_eq!(lines[1], "line2  ");
        assert_eq!(lines[2], "  line3  ");
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_err_not_subdirectory(_tmp_dir: TempDir) {
        let path = PathBuf::from("/etc/test.txt");
        let err = read_lines(path, 1, 2).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Path must be a sub-directory of the current working directory"
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_err_out_of_index(_tmp_dir: TempDir) {
        Write::write_all(
            &mut File::create("test.txt").unwrap(),
            b"line1\nline2\nline3",
        )
        .unwrap();
        let path = PathBuf::from("test.txt");
        let start_line = 1;
        let end_line = 4;
        let err = read_lines(path, start_line, end_line).await.unwrap_err();
        assert_eq!(err.to_string(), "End line is out of index");
    }
}
