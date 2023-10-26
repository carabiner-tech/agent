use regex::Regex;

#[derive(Debug)]
pub struct Hunk {
    pub original_start: usize,
    pub original_len: usize,
    pub new_start: usize,
    pub new_len: usize,
    pub start_context_lines: Vec<String>,
    pub changes: Vec<DiffLine>,
    pub changes_have_leading_space: bool,
}

lazy_static::lazy_static! {
    static ref HUNK_HEADER_RE: Regex =
        Regex::new(r"@@ \-(\d+),?(\d*) \+(\d+),?(\d*) @@").unwrap();
}

impl Hunk {
    pub fn parse(input: &str) -> Result<Hunk, ParseError> {
        let lines: Vec<&str> = input.lines().collect();
        Self::parse_lines(&lines)
    }

    pub fn parse_lines(lines: &[&str]) -> Result<Hunk, ParseError> {
        // Parse hunk header
        let header = lines.first().ok_or(ParseError::MissingHeader)?;
        let captures = HUNK_HEADER_RE
            .captures(header)
            .ok_or(ParseError::InvalidHunkHeader)?;

        let original_start = captures
            .get(1)
            .map(|m| m.as_str().parse().unwrap())
            .unwrap_or(0);
        let original_len = captures
            .get(2)
            .map_or(1, |m| m.as_str().parse().unwrap_or(1));
        let new_start = captures
            .get(3)
            .map(|m| m.as_str().parse().unwrap())
            .unwrap_or(0);
        let new_len = captures
            .get(4)
            .map_or(1, |m| m.as_str().parse().unwrap_or(1));

        // Parse diff lines
        let mut start_context_lines = Vec::new();
        let mut changes = Vec::new();
        let mut past_start_context = false;
        let mut changes_have_leading_space = true;
        for line in lines[1..].iter() {
            match line.chars().next() {
                Some('+') => {
                    past_start_context = true;
                    // For adding a new empty line, will just be "+", maybe "+ " as well?
                    // When adding lines with content, should be "+ <content>"
                    match line.len() {
                        1 => changes.push(DiffLine::Added("".to_string())),
                        _ => {
                            // check if the second character is not a space
                            if line.chars().nth(1) != Some(' ') {
                                changes_have_leading_space = false;
                            }
                            changes.push(DiffLine::Added(line[1..].to_string()))
                        }
                    }
                }
                Some('-') => {
                    past_start_context = true;
                    changes.push(DiffLine::Removed(line[1..].to_string()))
                }
                Some(' ') | None if past_start_context => {
                    changes.push(DiffLine::Unchanged(line[1..].to_string()))
                }
                Some(' ') | None => {
                    start_context_lines.push(line[1..].to_string());
                }
                _ => break, // exit on non-diff line
            }
        }

        Ok(Hunk {
            original_start,
            original_len,
            new_start,
            new_len,
            start_context_lines,
            changes,
            changes_have_leading_space,
        })
    }

    pub fn apply(&self, original_lines: &mut Vec<String>) -> Result<(), ApplyError> {
        let mut index = match self.start_context_lines.is_empty() {
            true => 0, // if there are no context lines, start at index 0 (top of file)
            false => self
                .find_context_position(original_lines)
                .ok_or(ApplyError::ContextNotFound)?,
        };

        // Now we start applying changes. But first skip over the start context lines
        index += self.start_context_lines.len();

        // Apply the changes
        for change in &self.changes {
            match change {
                DiffLine::Added(line) => {
                    if self.changes_have_leading_space {
                        original_lines.insert(index, line[1..].to_string());
                    } else {
                        original_lines.insert(index, line.to_string());
                    }
                    index += 1;
                }
                DiffLine::Removed(expected_line) => {
                    let removed_line = original_lines.remove(index);
                    if removed_line.trim() != expected_line.as_str().trim() {
                        return Err(ApplyError::RemovedLineMismatch);
                    }
                }
                DiffLine::Unchanged(_) => {
                    index += 1;
                }
            }
        }

        Ok(())
    }

    pub fn find_context_position(&self, original_lines: &[String]) -> Option<usize> {
        let context = &self.start_context_lines[0];
        // First check the start line itself
        if original_lines.get(self.original_start) == Some(context) {
            return Some(self.original_start);
        }

        let mut distance = 1;

        loop {
            let idx_earlier = self.original_start.checked_sub(distance); // using checked_sub to handle underflow
            let idx_after = self.original_start + distance;

            // Try the line below
            if let Some(idx) = idx_earlier {
                if original_lines.get(idx) == Some(context) {
                    return Some(idx);
                }
            }

            // Try the line above
            if let Some(line) = original_lines.get(idx_after) {
                if line == context {
                    return Some(idx_after);
                }
            }

            // If we've tried all lines, break
            if idx_earlier.is_none() && idx_after >= original_lines.len() {
                break;
            }

            distance += 1;
        }

        None
    }
}

#[derive(Debug)]
pub enum DiffLine {
    Added(String),
    Removed(String),
    Unchanged(String),
}

#[derive(Debug)]
pub enum ParseError {
    MissingHeader,
    InvalidHunkHeader,
    InvalidLine,
    NormalDiff,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ParseError::MissingHeader => write!(f, "Expected hunk header, found nothing"),
            ParseError::InvalidHunkHeader => write!(f, "Failed to match hunk header"),
            ParseError::InvalidLine => write!(f, "Invalid line in hunk"),
            ParseError::NormalDiff => {
                write!(f, "Normal diff not supported, use unified diff format")
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
pub enum ApplyError {
    ContextNotFound,
    OutOfBounds,
    RemovedLineMismatch,
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ApplyError::ContextNotFound => {
                write!(f, "Failed to find the context position for the hunk")
            }
            ApplyError::OutOfBounds => write!(f, "Hunk is out of bounds"),
            ApplyError::RemovedLineMismatch => write!(
                f,
                "Mismatch between the expected removed line and the actual line in the file"
            ),
        }
    }
}

impl std::error::Error for ApplyError {}
