use std::borrow::Cow;
use std::collections::LinkedList;

use anyhow::{Ok, Result};
use oxc::ast::ast::Statement;
use oxc::parser::ParserReturn;
use oxc::span::{GetSpan, Span};
use ropey::Rope;

use crate::config::Configuration;
use crate::re;

use super::{ImportElement, LineSpan, Matcher, OtherElement, ProgramParts};

pub struct Formatter<'a> {
    pub config: &'a Configuration,
    pub src: &'a str,
    pub rope: Rope,
    pub ast: ParserReturn<'a>,
}

impl<'a> Formatter<'a> {
    pub fn format(self) -> Result<String> {
        let parts = self.extract_imports();

        let mut output = self.rope.clone();

        // Remove from bottom to top so that indexing will not be a mess
        for element in parts.imports.iter().rev() {
            remove_span(&mut output, element.span);

            for comment in element.comments.iter().rev() {
                remove_span(&mut output, comment.span);
            }
        }

        let groups = self.organize(parts.imports);

        let pos = self.rope.byte_to_char(parts.preamable.end as usize);
        for group in groups.iter().rev() {
            output.insert(pos, "\n");
            for element in group.iter().rev() {
                // Insert the element after preamable and before the elements previously inserted
                output.insert(pos, "\n");
                output.insert(pos, element.span.source_text(self.src));

                for comment in element.comments.iter().rev() {
                    output.insert(pos, "\n");
                    output.insert(pos, comment.span.source_text(self.src));
                }
            }
        }

        Ok(output.to_string())
    }

    pub fn extract_imports(&'a self) -> ProgramParts<'a> {
        let mut parts = ProgramParts {
            preamable: self.get_preamable_span(),
            imports: LinkedList::new(),
            body: Vec::with_capacity(self.ast.program.body.len()),
        };

        let mut last_end = parts.preamable.end;

        for statement in &self.ast.program.body {
            let span = statement.span();
            let lines = LineSpan::find(&self.rope, span);

            let mut comments_before = self.get_comments(last_end, span.start);

            if let Statement::ImportDeclaration(decl) = statement {
                let comments = self.pull_related_comments(&mut comments_before, statement);

                parts.imports.push_back(ImportElement {
                    span,
                    comments,
                    decl,
                });
            }

            parts.body.extend(comments_before);

            match statement {
                Statement::ImportDeclaration(_) => {}
                Statement::TSModuleDeclaration(_) => {
                    parts.body.push(OtherElement {
                        span,
                        lines,
                        is_module: true,
                    });
                }
                _ => parts.body.push(OtherElement {
                    span,
                    lines,
                    is_module: false,
                }),
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

        Span::new(
            self.ast.program.span.start,
            related
                .first()
                .map(|c| c.span.start)
                .unwrap_or(first.span().start),
        )
    }

    fn get_comments(&self, start: u32, end: u32) -> Vec<OtherElement> {
        self.ast
            .trivias
            .comments_range(start..end)
            .map(|comment| OtherElement::from_comment(&self.rope, comment))
            .collect()
    }

    fn pull_related_comments(
        &self,
        comments: &mut Vec<OtherElement>,
        statement: &Statement,
    ) -> Vec<OtherElement> {
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

fn remove_span(rope: &mut Rope, span: Span) {
    // Convert byte index to char index
    let start = rope.byte_to_char(span.start as usize);
    let end = rope.byte_to_char(span.end as usize);

    // Remove the statement from the rope
    rope.remove(start..end);

    // Remove the line end if it has became empty
    let line = rope.char_to_line(start);
    let line_content: Cow<str> = rope.line(line).into();
    if re!(r"^\s*$").is_match(&line_content) {
        rope.remove(rope.line_to_char(line)..rope.line_to_char(line + 1));
    }
}
