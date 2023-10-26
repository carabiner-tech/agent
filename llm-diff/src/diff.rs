use crate::{ApplyError, Hunk, ParseError};
use regex::Regex;

#[derive(Debug)]
pub struct FileDiff {
    pub original_filename: Option<String>,
    pub new_filename: Option<String>,
    pub hunks: Vec<Hunk>,
}

lazy_static::lazy_static! {
    static ref NORMAL_DIFF_HEADER: Regex = Regex::new(r"^\d+(,\d+)?[acd]\d+(,\d+)?").unwrap();
}

impl FileDiff {
    pub fn parse(input: &str) -> Result<FileDiff, ParseError> {
        let lines_iter = input.lines();

        let mut original_filename = None;
        let mut new_filename = None;
        let mut hunks = Vec::new();

        let mut current_hunk_lines = Vec::new();

        for line in lines_iter {
            if let Some(orig_header) = line.strip_prefix("--- ") {
                original_filename = Some(orig_header.to_string());
            } else if let Some(new_header) = line.strip_prefix("+++ ") {
                new_filename = Some(new_header.to_string());
            } else if line.starts_with("@@ ") {
                // If we already have lines in current_hunk_lines, this means we found a new hunk,
                // so we parse the previous hunk and start a new one
                if !current_hunk_lines.is_empty() {
                    let hunk = Hunk::parse_lines(&current_hunk_lines)?;
                    hunks.push(hunk);
                    current_hunk_lines = Vec::new();
                }
                current_hunk_lines.push(line);
            } else if current_hunk_lines.is_empty() || NORMAL_DIFF_HEADER.is_match(line) {
                return Err(ParseError::NormalDiff);
            } else {
                if current_hunk_lines.is_empty() {
                    return Err(ParseError::MissingHeader);
                }
                current_hunk_lines.push(line);
            }
        }

        // Parse any remaining hunk
        if !current_hunk_lines.is_empty() {
            let hunk = Hunk::parse_lines(&current_hunk_lines)?;
            hunks.push(hunk);
        }

        Ok(FileDiff {
            original_filename,
            new_filename,
            hunks,
        })
    }

    pub fn apply(&self, original_lines: &[String]) -> Result<Vec<String>, ApplyError> {
        let mut file_lines = original_lines.to_vec();

        for hunk in &self.hunks {
            match hunk.apply(&mut file_lines) {
                Ok(_) => (),
                Err(err) => return Err(err),
            }
        }

        Ok(file_lines)
    }
}
