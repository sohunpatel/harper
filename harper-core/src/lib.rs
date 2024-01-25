#![allow(dead_code)]

mod document;
mod lexing;
mod linting;
pub mod parsers;
mod span;
mod spell;
mod token;

pub use document::Document;
pub use linting::LintSet;
pub use linting::{Lint, LintKind, Linter, Suggestion};
pub use span::Span;
pub use spell::Dictionary;
pub use token::{FatToken, Punctuation, Token, TokenKind, TokenStringExt};
