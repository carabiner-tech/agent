use std::{error::Error, path::PathBuf};

pub(crate) async fn ensure_relative(path: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    match path.is_relative() {
        true => Ok(path),
        false => match path.strip_prefix(std::env::current_dir()?) {
            Ok(relative_path) => Ok(relative_path.to_path_buf()),
            Err(_) => Err("Path must be a sub-directory of the current working directory".into()),
        },
    }
}

pub(crate) async fn read_lines(path: &PathBuf) -> Result<Vec<String>, Box<dyn Error>> {
    // Bubble up exception if file isn't found
    let content = tokio::fs::read_to_string(path).await?;

    // Gotcha here: .lines() will strip trailing \n so foo\nbar\nbaz is the same as foo\nbar\nbaz\n
    let has_trailing_newline = content.ends_with('\n');
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    if has_trailing_newline {
        lines.push("".into());
    }
    Ok(lines)
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
    async fn test_ensure_relative(_tmp_dir: TempDir) {
        let happy_path = ensure_relative(PathBuf::from("test.txt")).await;
        assert!(happy_path.is_ok());

        let unhappy_path = ensure_relative(PathBuf::from("/etc/test.txt")).await;
        assert!(unhappy_path.is_err());
        assert_eq!(
            unhappy_path.unwrap_err().to_string(),
            "Path must be a sub-directory of the current working directory"
        );
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_read_lines_no_trailing_line(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3").unwrap();
        let path = PathBuf::from("test.txt");
        let lines = read_lines(&path).await.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[2], "line3")
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_read_lines_with_trailing_lines(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"line1\nline2\nline3\n\n").unwrap();
        let path = PathBuf::from("test.txt");
        let lines = read_lines(&path).await.unwrap();
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[2], "line3");
        assert_eq!(lines[3], "");
        assert_eq!(lines[4], "")
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_preserves_inline_whitespace(_tmp_dir: TempDir) {
        let mut f = File::create("test.txt").unwrap();
        f.write_all(b"  line1\nline2  \n  line3  ").unwrap();
        let path = PathBuf::from("test.txt");
        let lines = read_lines(&path).await.unwrap();
        assert_eq!(lines[0], "  line1");
        assert_eq!(lines[1], "line2  ");
        assert_eq!(lines[2], "  line3  ");
    }
}
