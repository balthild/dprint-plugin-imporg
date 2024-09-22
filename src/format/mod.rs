use std::path::Path;

use anyhow::{bail, Result};
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::SourceType;
use ropey::Rope;

use crate::config::Configuration;

mod element;
mod formatter;
mod line;
mod matcher;
mod program;

use element::*;
use formatter::*;
use line::*;
use matcher::*;
use program::*;

pub fn format_source(config: &Configuration, path: &Path, src: &[u8]) -> Result<Vec<u8>> {
    let src = std::str::from_utf8(src)?;
    let rope = Rope::from_str(src);

    let alloc = Allocator::default();
    let typ = SourceType::from_path(path)?;

    let ast = Parser::new(&alloc, src, typ).parse();
    if !ast.errors.is_empty() {
        bail!("source code contains errors");
    }

    let formatter = Formatter {
        config,
        src,
        rope,
        ast,
    };

    let output = formatter.format()?;

    Ok(output.into_bytes())
}
