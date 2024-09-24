use std::cmp::Ordering;

use oxc::ast::ast::{ImportDeclaration, TSModuleDeclaration, TSModuleDeclarationBody};
use oxc::ast::Comment;
use oxc::span::Span;
use ropey::Rope;

use super::LineSpan;

#[derive(Debug)]
pub struct CommentElement {
    pub span: Span,
    pub lines: LineSpan,
}

impl CommentElement {
    pub fn from_ast(rope: &Rope, comment: &Comment) -> Self {
        let span = comment.real_span();
        let lines = LineSpan::find(rope, span);
        Self { span, lines }
    }
}

#[derive(Debug)]
pub struct ImportElement<'a> {
    pub span: Span,
    pub comments: Vec<CommentElement>,
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

#[derive(Debug)]
pub struct ModuleElement {
    pub body: Span,
}

impl ModuleElement {
    pub fn from_ast(decl: &TSModuleDeclaration) -> Option<Self> {
        decl.body.as_ref().and_then(|body| match body {
            TSModuleDeclarationBody::TSModuleBlock(it) => Some(ModuleElement {
                body: it.span.shrink(1),
            }),
            TSModuleDeclarationBody::TSModuleDeclaration(it) => ModuleElement::from_ast(it),
        })
    }
}
