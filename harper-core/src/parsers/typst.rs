use super::{Parser, PlainEnglish};
use crate::{Span, Token, TokenKind};

use typst_syntax::{LinkedNode, Side, SyntaxKind};

pub struct Typst;

impl Typst {}

impl Parser for Typst {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let source_str: String = source.iter().collect();
        let typst_node = typst_syntax::parse(&source_str);
        let root_node = LinkedNode::new(&typst_node);

        let mut tokens = Vec::new();

        // To find all the nodes, we can search for nodes by their offset from the beginning of
        // the source file. We know to stop searching when the node comes to the end of the root
        // source node's length.
        let source_node_length = typst_node.len();
        let mut cursor: usize = 0;
        // In some situations, we want to disable linting for a while (i.e. in match mode)
        let mut disable_linting: Option<SyntaxKind> = None;
        while cursor < source_node_length {
            let current_node = root_node.leaf_at(cursor, Side::After).unwrap();
            let range = current_node.range();

            if disable_linting.is_some_and(|t| t == current_node.kind()) {
                // We have reached the matching node that will signal that we can start linting
                // again. We still have to mark this node as Unlintable though.
                disable_linting = None;
                tokens.push(Token {
                    span: Span::new_with_len(cursor, range.end),
                    kind: TokenKind::Unlintable,
                });
                cursor = range.end;
                continue;
            } else if disable_linting.is_some() {
                // We still want to keep linting disabled, but we need to mark this node as
                // Unlintable.
                tokens.push(Token {
                    span: Span::new_with_len(cursor, range.end),
                    kind: TokenKind::Unlintable,
                });
                cursor = range.end;
                continue;
            }

            match current_node.kind() {
                SyntaxKind::Text => {
                    // For all text we can just use the standard English parser
                    let mut engligh_parser = PlainEnglish;

                    let mut new_tokens = engligh_parser.parse(&source[range.start..range.end]);
                    // We need to update the spans of each token with the offset of the node from
                    // the beginning of the source file
                    new_tokens
                        .iter_mut()
                        .for_each(|token| token.span.push_by(range.start));

                    tokens.append(&mut new_tokens);
                }
                SyntaxKind::Space | SyntaxKind::Parbreak => {
                    // The Typst syntax uses a space as a representation of any whitespace. This
                    // be used in scenarios where you need to separate different operators or text.
                    let count = current_node.text().matches("\n").count();
                    if count > 0 {
                        tokens.push(Token {
                            span: Span::new(cursor, range.end),
                            // We want to add an additional newline to signify and end of a linting
                            // section
                            kind: TokenKind::Newline(count + 1),
                        });
                    } else {
                        tokens.push(Token {
                            span: Span::new(cursor, range.end),
                            kind: TokenKind::Space(range.end - cursor),
                        })
                    }
                }
                SyntaxKind::Dollar | SyntaxKind::RawDelim => {
                    disable_linting = Some(current_node.kind());
                    tokens.push(Token {
                        span: Span::new_with_len(cursor, range.end),
                        kind: TokenKind::Unlintable,
                    });
                }
                // All markers are unlintable
                SyntaxKind::ListMarker
                | SyntaxKind::HeadingMarker
                | SyntaxKind::Underscore
                | SyntaxKind::Star
                | SyntaxKind::LeftBracket
                | SyntaxKind::RightBracket => tokens.push(Token {
                    span: Span::new(cursor, range.end),
                    kind: TokenKind::Unlintable,
                }),
                _ => tokens.push(Token {
                    span: Span::new(cursor, range.end),
                    kind: TokenKind::Unlintable,
                }),
            }

            // Mover cursor to end of node
            cursor = range.end;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::super::StrParser;
    use super::Typst;
    use crate::{Punctuation, Token, TokenKind};

    fn all_chars_tokenized(tokens: &Vec<Token>) -> bool {
        let mut cursor = 0;
        for token in tokens {
            if token.span.start > cursor + 1 {
                return false;
            }

            cursor = token.span.end;
        }
        true
    }

    #[test]
    fn regular_text() {
        let source = "This is a test.";

        let tokens = Typst.parse_str(source);
        assert!(all_chars_tokenized(&tokens));
        assert!(tokens.last().unwrap().span.end == source.len());

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();
        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Period)
            ]
        ));
    }

    #[test]
    fn headings() {
        let source = "= This is a heading";

        let tokens = Typst.parse_str(source);
        assert!(all_chars_tokenized(&tokens));
        assert!(tokens.last().unwrap().span.end == source.len());

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();
        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
            ]
        ));
    }

    #[test]
    fn lists() {
        let source = "- test one\n- test two\n- test three";

        let tokens = Typst.parse_str(source);
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.kind.is_newline())
                .count(),
            2
        );
        assert!(all_chars_tokenized(&tokens));

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();
        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Newline(2),
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Newline(2),
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
            ]
        ));
    }

    #[test]
    fn inline_math() {
        let source = "Let $x = 27$.";

        let tokens = Typst.parse_str(source);
        assert_eq!(
            tokens
                .iter()
                .filter(|token| token.kind.is_unlintable())
                .count(),
            7
        );
        assert!(all_chars_tokenized(&tokens));
        assert!(tokens.last().unwrap().span.end == source.len());

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();
        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Punctuation(Punctuation::Period),
            ]
        ));
    }
}
