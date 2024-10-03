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
    return rope.line(line).chars().all(char::is_whitespace);
}
