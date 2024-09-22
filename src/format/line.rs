use oxc::span::Span;
use ropey::Rope;

/// Inclusive. Zero-indexed.
/// Use `u32` because oxc uses `u32`. See `oxc::span::Span`.
#[derive(Clone, Copy)]
pub struct LineSpan {
    pub start: u32,
    pub end: u32,
}

impl LineSpan {
    pub fn find(rope: &Rope, span: Span) -> Self {
        let start = rope.byte_to_line(span.start as usize) as u32;
        let end = rope.byte_to_line(span.end as usize) as u32;

        Self { start, end }
    }

    pub fn tightly_following(self, before: LineSpan) -> bool {
        matches!(self.start - before.end, 0..=1)
    }
}

impl std::fmt::Debug for LineSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LineSpan[{}, {}]", self.start, self.end)
    }
}
