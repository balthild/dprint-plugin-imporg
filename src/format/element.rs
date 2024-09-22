use std::cmp::Ordering;

use oxc::ast::ast::ImportDeclaration;
use oxc::ast::Comment;
use oxc::span::Span;
use ropey::Rope;

use super::LineSpan;

#[derive(Debug)]
pub struct OtherElement {
    pub span: Span,
    pub lines: LineSpan,
    // TODO: format recursively
    pub is_module: bool,
}

impl OtherElement {
    pub fn from_comment(rope: &Rope, comment: &Comment) -> Self {
        let span = Span::new(comment.real_span_start(), comment.real_span_end());
        Self {
            span,
            lines: LineSpan::find(rope, span),
            is_module: false,
        }
    }
}

#[derive(Debug)]
pub struct ImportElement<'a> {
    pub span: Span,
    pub comments: Vec<OtherElement>,
    pub decl: &'a ImportDeclaration<'a>,
}

impl<'a> ImportElement<'a> {
    pub fn module(&self) -> &'a str {
        self.decl.source.value.as_str()
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        let left = self.module();
        let right = other.module();

        match left.to_lowercase().cmp(&right.to_lowercase()) {
            Ordering::Equal => left.cmp(right),
            ord => ord,
        }
    }
}
