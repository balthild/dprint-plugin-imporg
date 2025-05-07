use std::path::Path;

use anyhow::{bail, Result};
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::{SourceType, Span};
use ropey::Rope;

use crate::config::Configuration;

mod editing;
mod element;
mod formatter;
mod line;
mod matcher;
mod program;

use editing::*;
use element::*;
use formatter::*;
use line::*;
use matcher::*;
use program::*;

pub fn format_source(
    config: &Configuration,
    path: &Path,
    src: &str,
    inner: Option<Span>,
) -> Result<Rope> {
    let rope = Rope::from_str(src);

    let alloc = Allocator::default();
    let typ = SourceType::from_path(path)?;

    let ast = Parser::new(&alloc, src, typ).parse();
    if !ast.errors.is_empty() {
        for error in ast.errors {
            let mut message = error.message.to_string();

            if let Some(labels) = &error.labels {
                for label in labels {
                    let offset = label.offset();
                    let line = rope.byte_to_line(offset);
                    let column = offset - rope.line_to_byte(line);
                    message.push_str(&format!(
                        "\n    at {}:{}:{}",
                        path.display(),
                        line + 1,
                        column + 1
                    ));
                }
            }

            if let Some(help) = &error.help {
                message.push_str(&format!("\n    {help}"));
            }

            crate::utils::log(&message);
        }

        bail!("source code contains errors");
    }

    let formatter = Formatter {
        config,
        src,
        rope,
        ast,
        inner,
    };

    let mut ret = formatter.format()?;

    for module in ret.submodules.into_iter().rev() {
        let range = module.outer.start as usize..module.outer.end as usize;
        let src = ret.output.byte_slice(range).to_string();
        let output = format_source(config, path, &src, Some(module.inner))?;

        let start = ret.output.byte_to_char(module.outer.start as usize);
        let end = ret.output.byte_to_char(module.outer.end as usize);
        ret.output.remove(start..end);
        ret.output.insert(start, &output.to_string());
    }

    Ok(ret.output)
}
