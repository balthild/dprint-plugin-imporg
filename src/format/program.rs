use std::collections::LinkedList;

use oxc::span::Span;

use super::{CommentElement, ImportElement, ModuleElement};

#[derive(Debug)]
pub struct ProgramParts<'a> {
    pub preamable: Span,
    pub imports: LinkedList<ImportElement<'a>>,
    pub comments: Vec<CommentElement>,
    pub submodules: Vec<ModuleElement>,
}
