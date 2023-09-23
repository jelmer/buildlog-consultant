use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Iterator;

// Define the struct for SbuildLogSection.
pub struct SbuildLogSection {
    pub title: Option<String>,
    pub offsets: (usize, usize),
    pub lines: Vec<String>,
}

// Define the function to parse sbuild log.
pub fn parse_sbuild_log<R: BufRead>(mut reader: R) -> impl Iterator<Item = SbuildLogSection> {
    let mut begin_offset = 1;
    let mut lines = Vec::new();
    let mut title: Option<String> = None;

    // Separator line (78 '-' characters, bookended by '+').
    let sep = "+".to_string() + &"-".repeat(78) + "+";
    let mut lineno = 0;

    // We'll store our sections in this Vec and return it as an iterator at the end.
    let mut sections = Vec::new();

    loop {
        let mut line = String::new();

        // Read a line from the file. Break if EOF.
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }

        lineno += 1;

        // Trim trailing whitespace and newline characters.
        let line_trimmed = line.trim().to_string();

        if line_trimmed == sep {
            // Read next two lines
            let mut l1 = String::new();
            let mut l2 = String::new();

            reader.read_line(&mut l1).unwrap();
            reader.read_line(&mut l2).unwrap();

            lineno += 2;

            // Trim trailing whitespace and newline characters.
            let l1_trimmed = l1.trim().to_string();
            let l2_trimmed = l2.trim().to_string();

            if l1.starts_with('|') && l1_trimmed.ends_with('|') && l2_trimmed == sep {
                let mut end_offset = lineno - 3;

                // Drop trailing empty lines
                while lines.last() == Some(&"\n".to_string()) {
                    lines.pop();
                    end_offset -= 1;
                }

                if !lines.is_empty() {
                    // The unwrap_or_else is to provide a default value in case 'title' is None.
                    sections.push(SbuildLogSection {
                        title: title.clone(),
                        offsets: (begin_offset, end_offset),
                        lines: lines.clone(),
                    });
                }

                title = Some(l1.trim_end()[1..l1.len() - 1].trim().to_string());
                lines.clear();
                begin_offset = lineno;
            } else {
                lines.push(line);
                lines.push(l1);
                lines.push(l2);
            }
        } else {
            lines.push(line);
        }
    }

    // Generate the final section.
    sections.push(SbuildLogSection {
        title,
        offsets: (begin_offset, lineno),
        lines,
    });

    // Return the sections as an iterator.
    sections.into_iter()
}
