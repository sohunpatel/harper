mod affix_replacement;
mod attribute_list;
mod error;
mod expansion;
mod matcher;
mod word_list;

pub use attribute_list::AttributeList;
use attribute_list::HumanReadableAttributeList;
pub use error::Error;

use self::word_list::parse_word_list;
pub use self::word_list::MarkedWord;

pub fn parse_default_word_list() -> Result<Vec<MarkedWord>, Error> {
    parse_word_list(include_str!("../../../dictionary.dict"))
}

pub fn parse_default_attribute_list() -> AttributeList {
    let human_readable: HumanReadableAttributeList =
        serde_json::from_str(include_str!("../../../affixes.json"))
            .expect("The built-in affix list should always be valid.");

    human_readable
        .to_normal()
        .expect("All expressions in the built-in attribute list should be valid.")
}

#[cfg(test)]
mod tests {
    use hashbrown::HashMap;
    use serde_json::json;

    use super::word_list::parse_word_list;
    use super::{parse_default_attribute_list, parse_default_word_list};
    use crate::spell::hunspell::attribute_list::HumanReadableAttributeList;
    use crate::{CharString, WordMetadata};

    pub const TEST_WORD_LIST: &str = "3\nhello\ntry/B\nwork/AB";

    #[test]
    fn correctly_expands_test_files() {
        let words = parse_word_list(TEST_WORD_LIST).unwrap();
        let attributes: HumanReadableAttributeList = serde_json::from_value(json!({
            "affixes": {
                "A": {
                    "suffix": false,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "re",
                        "condition": "."
                      }
                    ],
                    "adds_metadata": {
                      "kind": null,
                      "tense": null
                    }
                },
                "B": {
                    "suffix": true,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "ed",
                        "condition": "[^y]"
                      },
                      {
                        "remove": "y",
                        "add": "ied",
                        "condition": "y"
                      }
                    ],
                    "adds_metadata": {
                      "kind": null,
                      "tense": null
                    }
                }
            }
        }))
        .unwrap();
        let attributes = attributes.to_normal().unwrap();

        let mut expanded = HashMap::new();

        attributes.expand_marked_words(words, &mut expanded);
        let expanded: Vec<String> = expanded
            .into_iter()
            .map(|v| v.0.into_iter().collect())
            .collect();

        assert_eq!(
            expanded,
            vec!["hello", "tried", "reworked", "rework", "worked", "work", "try"]
        )
    }

    #[test]
    fn plural_giants() {
        let words = parse_word_list("1\ngiant/SM").unwrap();

        let attributes: HumanReadableAttributeList = serde_json::from_value(json!({
            "affixes": {
                "S": {
                    "suffix": true,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "y",
                        "add": "ies",
                        "condition": "[^aeiou]"
                      },
                      {
                        "remove": "",
                        "add": "s",
                        "condition": "[aeiou]y"
                      },
                      {
                        "remove": "",
                        "add": "s",
                        "condition": "[^sxzhy]"
                      }
                    ],
                    "adds_metadata": {
                      "kind": null,
                      "tense": null
                    }
                },
                "M": {
                    "suffix": true,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "'s",
                        "condition": "."
                      }
                    ],
                    "adds_metadata": {
                      "kind": null,
                      "tense": null
                    }
                }
            }
        }))
        .unwrap();
        let attributes = attributes.to_normal().unwrap();

        let mut expanded: HashMap<CharString, WordMetadata> = HashMap::new();

        attributes.expand_marked_words(words, &mut expanded);

        assert!(expanded.contains_key(&split("giants")))
    }

    fn build_expanded() -> HashMap<CharString, WordMetadata> {
        let words = parse_default_word_list().unwrap();
        let attributes = parse_default_attribute_list();

        let mut expanded = HashMap::new();

        attributes.expand_marked_words(words, &mut expanded);

        expanded
    }

    #[test]
    fn can_expand_default() {
        build_expanded();
    }

    #[test]
    fn expanded_contains_giants() {
        assert!(build_expanded().contains_key(&split("giants")));
    }

    #[test]
    fn expanded_contains_deallocate() {
        assert!(build_expanded().contains_key(&split("deallocate")));
    }

    fn split(text: &str) -> CharString {
        text.chars().collect()
    }
}
