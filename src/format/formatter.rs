use std::collections::LinkedList;

use anyhow::{Ok, Result};
use oxc::ast::ast::Statement;
use oxc::parser::ParserReturn;
use oxc::span::{GetSpan, Span};
use ropey::Rope;

use crate::config::Configuration;

use super::{
    remove_span, ChangedSpan, CommentElement, ImportElement, LineSpan, Matcher, ModuleElement,
    ProgramParts,
};

pub struct FormatterReturn {
    pub output: Rope,
    pub submodules: Vec<Span>,
}

pub struct Formatter<'a> {
    pub config: &'a Configuration,
    pub src: &'a str,
    pub rope: Rope,
    pub ast: ParserReturn<'a>,
}

impl<'a> Formatter<'a> {
    pub fn format(self) -> Result<FormatterReturn> {
        let parts = self.extract_parts();

        let mut submodules: Vec<_> = parts.submodules.into_iter().map(|m| m.body).collect();
        let mut output = self.rope.clone();

        // Remove from bottom to top so that indexing will not be a mess
        for element in parts.imports.iter().rev() {
            let removed = remove_span(&mut output, element.span);
            removed.update_spans(&mut submodules)?;

            for comment in element.comments.iter().rev() {
                let removed = remove_span(&mut output, comment.span);
                removed.update_spans(&mut submodules)?;
            }
        }

        let groups = self.organize(parts.imports);

        // Insert imports after preamable and before those previously inserted
        let pos = self.rope.byte_to_char(parts.preamable.end as usize);
        let mut inserted = ChangedSpan::empty(parts.preamable.end);
        for group in groups.iter().rev() {
            output.insert(pos, "\n");
            inserted.len += 1;

            for element in group.iter().rev() {
                output.insert(pos, "\n");
                output.insert(pos, element.span.source_text(self.src));
                inserted.len += element.span.size() as i64 + 1;

                for comment in element.comments.iter().rev() {
                    output.insert(pos, "\n");
                    output.insert(pos, comment.span.source_text(self.src));
                    inserted.len += comment.span.size() as i64 + 1;
                }
            }
        }

        inserted.update_spans(&mut submodules)?;

        Ok(FormatterReturn { output, submodules })
    }

    pub fn extract_parts(&'a self) -> ProgramParts<'a> {
        let mut parts = ProgramParts {
            preamable: self.get_preamable_span(),
            imports: LinkedList::new(),
            comments: vec![],
            submodules: vec![],
        };

        let mut last_end = parts.preamable.end;

        for statement in &self.ast.program.body {
            let span = statement.span();

            let mut comments_before = self.get_comments(last_end, span.start);

            if let Statement::ImportDeclaration(decl) = statement {
                let comments = self.pull_related_comments(&mut comments_before, statement);

                parts.imports.push_back(ImportElement {
                    span,
                    comments,
                    decl,
                });
            }

            parts.comments.extend(comments_before);

            if let Statement::TSModuleDeclaration(ref decl) = statement {
                if let Some(element) = ModuleElement::from_ast(decl) {
                    parts.submodules.push(element);
                };
            }

            last_end = span.end;
        }

        parts
    }

    fn organize(&self, mut imports: LinkedList<ImportElement<'a>>) -> Vec<Vec<ImportElement<'a>>> {
        let mut groups = Vec::with_capacity(self.config.groups.len() + 1);

        let matcher = Matcher::new(self.config);

        for group in &self.config.groups {
            let matched: Vec<_> = imports
                .extract_if(|element| matcher.matches_group(group, element))
                .collect();

            if !matched.is_empty() {
                groups.push(matched);
            }
        }

        if !imports.is_empty() {
            groups.push(imports.into_iter().collect());
        }

        for group in groups.iter_mut() {
            group.sort_by(|a, b| a.compare(b));
        }

        groups
    }

    fn get_preamable_span(&self) -> Span {
        let Some(first) = self.ast.program.body.first() else {
            return self.ast.program.span;
        };

        let mut comments = self.get_comments(self.ast.program.span.start, first.span().start);
        let related = self.pull_related_comments(&mut comments, first);

        let mut end = related
            .first()
            .map(|c| c.span.start)
            .unwrap_or(first.span().start);

        let end_line = self.rope.byte_to_line(end as usize);
        let end_line_start = self.rope.line_to_byte(end_line);
        let end_line_content = &self.src[end_line_start..end as usize];
        if end_line_content.chars().all(char::is_whitespace) {
            end = end_line_start as u32;
        }

        Span::new(self.ast.program.span.start, end)
    }

    fn get_comments(&self, start: u32, end: u32) -> Vec<CommentElement> {
        self.ast
            .trivias
            .comments_range(start..end)
            .map(|comment| CommentElement::from_ast(&self.rope, comment))
            .collect()
    }

    fn pull_related_comments(
        &self,
        comments: &mut Vec<CommentElement>,
        statement: &Statement,
    ) -> Vec<CommentElement> {
        let mut split_at = comments.len();
        let mut next_lines = LineSpan::find(&self.rope, statement.span());

        for comment in comments.iter().rev() {
            if !next_lines.tightly_following(comment.lines) {
                break;
            }

            split_at -= 1;
            next_lines = comment.lines;
        }

        comments.split_off(split_at)
    }
}
