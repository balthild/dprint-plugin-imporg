use anyhow::{bail, Ok, Result};
use oxc::span::Span;
use ropey::Rope;

#[derive(Debug)]
pub struct ChangedSpan {
    pub pos: u32,
    pub len: i64,
}

impl ChangedSpan {
    pub fn empty(pos: u32) -> Self {
        Self { pos, len: 0 }
    }

    pub fn removal(span: Span) -> Self {
        Self {
            pos: span.start,
            len: -(span.size() as i64),
        }
    }

    pub fn update_spans<'a>(&self, spans: impl IntoIterator<Item = &'a mut Span>) -> Result<()> {
        let start = self.pos;
        let end = self.pos as i64 - self.len;

        for span in spans {
            // crate::utils::debug_print(self);
            // crate::utils::debug_print(&span);

            if start < span.start && end <= span.start as i64 {
                // the edit is before the span
                // crate::utils::debug_print("before");
                span.start = (span.start as i64 + self.len) as u32;
                span.end = (span.end as i64 + self.len) as u32;
                continue;
            }

            if start >= span.start && end <= span.end as i64 {
                // The edit is inside the span
                // crate::utils::debug_print("inside");
                span.end = (span.end as i64 + self.len) as u32;
                continue;
            }

            if self.pos >= span.end {
                // The edit is after the span
                // crate::utils::debug_print("after");
                continue;
            }

            // The edit crosses the boundary of the span
            bail!("the formatter went wild");
        }

        Ok(())
    }
}

pub fn remove_span(rope: &mut Rope, span: Span) -> ChangedSpan {
    let mut removed = ChangedSpan::removal(span);

    // Convert byte index to char index
    let start = rope.byte_to_char(span.start as usize);
    let end = rope.byte_to_char(span.end as usize);

    // Remove the statement from the rope
    rope.remove(start..end);

    // Remove the entire line if it has became blank
    let line = rope.char_to_line(start);
    if line_is_blank(rope, line) {
        let line_start = rope.line_to_byte(line);
        let line_end = rope.line_to_byte(line + 1);
        let line_len = line_end - line_start;

        removed = ChangedSpan {
            pos: line_start as u32,
            len: -(span.size() as i64 + line_len as i64),
        };

        rope.remove(rope.line_to_char(line)..rope.line_to_char(line + 1));
    }

    removed
}

pub fn insert(rope: &mut Rope, pos: usize, text: &str) -> i64 {
    rope.insert(pos, text);
    text.len() as i64
}

pub fn line_is_blank(rope: &Rope, line: usize) -> bool {
    rope.line(line).chars().all(char::is_whitespace)
}
