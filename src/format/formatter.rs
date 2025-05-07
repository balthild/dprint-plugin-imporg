use std::collections::LinkedList;

use anyhow::{bail, Result};
use oxc::allocator::Vec as OxcVec;
use oxc::ast::ast::{Statement, TSModuleDeclaration, TSModuleDeclarationBody};
use oxc::ast::comments_range;
use oxc::parser::ParserReturn;
use oxc::span::{GetSpan, Span};
use ropey::Rope;

use crate::config::Configuration;

use super::{
    insert, line_is_blank, remove_span, ChangedSpan, CommentElement, ImportElement, LineSpan,
    Matcher, ModuleElement, ProgramParts,
};

pub struct FormatterReturn {
    pub output: Rope,
    pub submodules: Vec<ModuleElement>,
}

pub struct Formatter<'a> {
    pub config: &'a Configuration,
    pub src: &'a str,
    pub rope: Rope,
    pub ast: ParserReturn<'a>,
    pub inner: Option<Span>,
}

impl<'a> Formatter<'a> {
    pub fn format(self) -> Result<FormatterReturn> {
        let indent = self.detect_indent();
        let parts = self.extract_parts()?;

        let mut submodules = parts.submodules.clone();
        let mut output = self.rope.clone();

        // Remove from bottom to top so that indexing will not be a mess
        for element in parts.imports.iter().rev() {
            let removed = remove_span(&mut output, element.span);
            removed.update_spans(
                submodules
                    .iter_mut()
                    .flat_map(|x| [&mut x.inner, &mut x.outer]),
            )?;

            for comment in element.comments.iter().rev() {
                let removed = remove_span(&mut output, comment.span);
                removed.update_spans(
                    submodules
                        .iter_mut()
                        .flat_map(|x| [&mut x.inner, &mut x.outer]),
                )?;
            }
        }

        let groups = self.organize(parts.imports);

        // Insert imports after preamable and before those previously inserted
        let pos = self.rope.byte_to_char(parts.preamable.end as usize);
        let mut inserted = ChangedSpan::empty(parts.preamable.end);
        for group in groups.iter().rev() {
            let line = output.char_to_line(pos);
            if !line_is_blank(&output, line) {
                inserted.len += insert(&mut output, pos, "\n");
            }

            for element in group.iter().rev() {
                inserted.len += insert(&mut output, pos, "\n");
                inserted.len += insert(&mut output, pos, element.span.source_text(self.src));
                inserted.len += insert(&mut output, pos, &indent);

                for comment in element.comments.iter().rev() {
                    inserted.len += insert(&mut output, pos, "\n");
                    inserted.len += insert(&mut output, pos, comment.span.source_text(self.src));
                    inserted.len += insert(&mut output, pos, &indent);
                }
            }
        }

        inserted.update_spans(
            submodules
                .iter_mut()
                .flat_map(|x| [&mut x.inner, &mut x.outer]),
        )?;

        Ok(FormatterReturn { output, submodules })
    }

    fn detect_indent(&self) -> String {
        let mut indent = String::new();

        for line in self.src.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let line_indent: String = line
                .chars()
                .take_while(|&c| c == ' ' || c == '\t')
                .collect();

            if line_indent.is_empty() {
                return indent;
            }

            if indent.is_empty() || line_indent.len() < indent.len() {
                indent = line_indent;
            }
        }

        indent
    }

    fn extract_parts(&'a self) -> Result<ProgramParts<'a>> {
        let mut parts = ProgramParts {
            preamable: self.get_preamable_span()?,
            imports: LinkedList::new(),
            comments: vec![],
            submodules: vec![],
        };

        let mut last_end = parts.preamable.end;

        for statement in self.get_body()? {
            let span = statement.span();

            let mut comments_before = self.get_comments(last_end, span.start);

            match statement {
                Statement::ImportDeclaration(decl) => {
                    let comments = self.pull_related_comments(&mut comments_before, statement);

                    parts.imports.push_back(ImportElement {
                        span,
                        comments,
                        decl,
                    });
                }
                Statement::TSModuleDeclaration(decl) => {
                    if let Some(element) = ModuleElement::from_ast(decl) {
                        parts.submodules.push(element);
                    };
                }
                _ => {}
            }

            parts.comments.extend(comments_before);

            last_end = span.end;
        }

        Ok(parts)
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

    fn get_body(&self) -> Result<&OxcVec<'a, Statement<'a>>> {
        if self.inner.is_none() {
            return Ok(&self.ast.program.body);
        }

        fn extract_block<'s, 'a>(
            decl: &'s TSModuleDeclaration<'a>,
        ) -> Result<&'s OxcVec<'a, Statement<'a>>> {
            match decl.body {
                Some(TSModuleDeclarationBody::TSModuleBlock(ref it)) => Ok(&it.body),
                Some(TSModuleDeclarationBody::TSModuleDeclaration(ref it)) => extract_block(it),
                None => bail!("bug: expecting module declaration block"),
            }
        }

        match self.ast.program.body.as_slice() {
            [Statement::TSModuleDeclaration(decl)] => extract_block(decl),
            _ => bail!("bug: expecting exact one module declaration"),
        }
    }

    fn get_preamable_span(&self) -> Result<Span> {
        let Some(first) = self.get_body()?.first() else {
            return Ok(self.ast.program.span);
        };

        let mut comments = self.get_comments(self.ast.program.span.start, first.span().start);
        let related = self.pull_related_comments(&mut comments, first);

        let mut end = related
            .first()
            .map(|c| c.span.start)
            .unwrap_or(first.span().start);

        // If the last line of preamable is empty, shrink it to the line start and treat this line as body.
        // Needing this because `remove_span` may remove an entire line.
        let end_line = self.rope.byte_to_line(end as usize);
        let end_line_start = self.rope.line_to_byte(end_line);
        let end_line_content = &self.src[end_line_start..end as usize];
        if end_line_content.chars().all(char::is_whitespace) {
            end = end_line_start as u32;
        }

        Ok(Span::new(self.ast.program.span.start, end))
    }

    fn get_comments(&self, start: u32, end: u32) -> Vec<CommentElement> {
        comments_range(&self.ast.program.comments, start..end)
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
