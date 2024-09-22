use std::collections::LinkedList;

use oxc::span::Span;

use super::{ImportElement, OtherElement};

#[derive(Debug)]
pub struct ProgramParts<'a> {
    pub preamable: Span,
    pub imports: LinkedList<ImportElement<'a>>,
    pub body: Vec<OtherElement>,
}
