use std::borrow::Cow;

use anyhow::{bail, Ok, Result};
use oxc::span::Span;
use ropey::Rope;

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

    pub fn update_spans(&self, spans: &mut [Span]) -> Result<()> {
        for span in spans {
            if self.pos < span.start {
                span.start = (span.start as i64 + self.len) as u32;
                span.end = (span.end as i64 + self.len) as u32;
            } else if self.pos >= span.start {
                // The edit does not affect this span
            } else {
                bail!("the formatter went wild");
            }
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

    // Remove the entire line if it has became empty
    let line = rope.char_to_line(start);
    let line_content: Cow<str> = rope.line(line).into();
    if line_content.chars().all(char::is_whitespace) {
        let line_start = rope.line_to_byte(line);
        let line_end = rope.line_to_byte(line + 1);
        removed = ChangedSpan {
            pos: line_start as u32,
            len: -(span.size() as i64 + line_end as i64 - line_start as i64),
        };

        rope.remove(rope.line_to_char(line)..rope.line_to_char(line + 1));
    }

    removed
}
