use super::dom::node::{Node, NodeType};
use combine::{
    choice,
    error::StreamError,
    error::StringStreamError,
    many, many1, optional,
    parser::char::{self, letter, newline, space},
    sep_by, sep_end_by, ParseError, Parser, Stream,
};
use thiserror::Error;

// style
#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Stylesheet { rules: rules }
    }
}

// 规则
#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        self.selectors.iter().any(|s| s.matches(n))
    }
}

pub type Selector = SimpleSelector;

// 选择器类型
#[derive(Debug, PartialEq)]
pub enum SimpleSelector {
    // 通用选择器
    UniversalSelector,
    // tag选择器
    TypeSelector {
        tag_name: String,
    },
    // 方法选择器
    AttributeSelector {
        tag_name: String,
        op: AttributeSelectorOp,
        attribute: String,
        value: String,
    },
    // 类型选择器
    ClassSelector {
        class_name: String,
    },
}

#[derive(Debug, PartialEq)]
pub enum AttributeSelectorOp {
    Eq,      // =
    Contain, // ~=
}

impl SimpleSelector {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        match self {
            SimpleSelector::UniversalSelector => true,
            SimpleSelector::TypeSelector { tag_name } => match n.node_type {
                NodeType::Element(ref e) => e.tag_name.as_str() == tag_name,
                _ => false,
            },
            SimpleSelector::AttributeSelector {
                tag_name,
                op,
                attribute,
                value,
            } => match n.node_type {
                NodeType::Element(ref e) => {
                    e.tag_name.as_str() == tag_name
                        && match op {
                            AttributeSelectorOp::Eq => e.attributes.get(attribute) == Some(value),
                            AttributeSelectorOp::Contain => e
                                .attributes
                                .get(attribute)
                                .map(|value| {
                                    value
                                        .split_ascii_whitespace()
                                        .find(|v| v == value)
                                        .is_some()
                                })
                                .unwrap_or(false),
                        }
                }
                _ => false,
            },
            SimpleSelector::ClassSelector { class_name } => match n.node_type {
                NodeType::Element(ref e) => e.attributes.get("class") == Some(class_name),
                _ => false,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: CSSValue,
}

// css value可以是 ’flex‘这样的字符串，也可以是10em这样的数字
#[derive(Debug, PartialEq, Clone)]
pub enum CSSValue {
    Keyword(String),
    Length((usize, Unit)),
}

// 单位，仅实现em
#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Em,
}

#[derive(Error, Debug, PartialEq)]
pub enum CSSParseError {
    #[error("failed to parse; {0}")]
    InvalidResourceError(StringStreamError),
}
// 得到Stylesheet
pub fn parse(raw: String) -> Result<Stylesheet, CSSParseError> {
    rules()
        .parse(raw.as_str())
        .map(|(rules, _)| Stylesheet::new(rules))
        .map_err(|e| CSSParseError::InvalidResourceError(e))
}

// 空格和换行符
fn whitespaces<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many::<String, _, _>(space().or(newline()))
}
// 获取所有css规则
fn rules<Input>() -> impl Parser<Input, Output = Vec<Rule>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (whitespaces(), many(rule().skip(whitespaces()))).map(|(_, rules)| rules)
}
// 获取一个css规则
fn rule<Input>() -> impl Parser<Input, Output = Rule>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        // 获取选择器
        selectors().skip(whitespaces()),
        char::char('{').skip(whitespaces()),
        // 获取属性
        declarations().skip(whitespaces()),
        char::char('}'),
    )
        .map(|(selectors, _, declarations, _)| Rule {
            selectors: selectors,
            declarations,
        })
}

// 解析选择器, .class or .class_1, .class_2
fn selectors<Input>() -> impl Parser<Input, Output = Vec<Selector>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by(
        selector().skip(whitespaces()),
        char::char(',').skip(whitespaces()),
    )
}

fn selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    simple_selector()
}

// 解析选择器
fn simple_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // 通用选择器以*开头
    let universal_selector = char::char('*').map(|_| SimpleSelector::UniversalSelector);
    // class选择器以.开头
    let class_selector =
        (char::char('.'), many1(letter())).map(|(_, class_name)| SimpleSelector::ClassSelector {
            class_name: class_name,
        });
    // 属性选择器需要先解析出tag,再解析出属性
    let type_or_attribute_selector = (
        // tag
        many1(letter()).skip(whitespaces()),
        // [xxx=xxx] or [xxx~=xxx]
        optional((
            char::char('[').skip(whitespaces()),
            many1(letter()),
            choice((char::string("="), char::string("~="))),
            many1(letter()),
            char::char(']'),
        )),
    )
        .and_then(|(tag_name, opts)| match opts {
            Some((_, attribute, op, value, _)) => {
                let op = match op {
                    "=" => AttributeSelectorOp::Eq,
                    "~=" => AttributeSelectorOp::Contain,
                    _ => {
                        return Err(<Input::Error as combine::error::ParseError<
                            char,
                            Input::Range,
                            Input::Position,
                        >>::StreamError::message_static_message(
                            "invalid attribute selector op",
                        ))
                    }
                };
                Ok(SimpleSelector::AttributeSelector {
                    tag_name: tag_name,
                    attribute: attribute,
                    op: op,
                    value: value,
                })
            }
            None => Ok(SimpleSelector::TypeSelector { tag_name: tag_name }),
        });

    choice((
        universal_selector,
        class_selector,
        type_or_attribute_selector,
    ))
}

// 匹配css每一项和；
fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by(
        declaration().skip(whitespaces()),
        char::char(';').skip(whitespaces()),
    )
}

// 匹配css的每一项
fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // 匹配 key, :, css_value
    (
        many1(letter()).skip(whitespaces()),
        char::char(':').skip(whitespaces()),
        css_value(),
    )
        .map(|(k, _, v)| Declaration { name: k, value: v })
}

// 解析css的value
fn css_value<Input>() -> impl Parser<Input, Output = CSSValue>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // 匹配字母
    let keyword = many1(letter()).map(|s| CSSValue::Keyword(s));
    // 匹配数字加em
    let length = (
        many1(char::digit()).map(|s: String| s.parse::<usize>().unwrap()),
        char::string("em"),
    )
        .map(|(num, _unit)| CSSValue::Length((num, Unit::Em)));
    choice((keyword, length))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stylesheet() {
        assert_eq!(
            parse("div[display=flex] { background: red; width: 1em } .rule { backgound: none;  }".to_string()),
            Ok(Stylesheet::new(vec![
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "div".to_string(),
                        attribute: "display".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "flex".to_string()
                    }],
                    declarations: vec![
                        Declaration {
                            name: "background".to_string(),
                            value: CSSValue::Keyword("red".to_string())
                        },
                        Declaration {
                            name: "width".to_string(),
                            value: CSSValue::Length((1, Unit::Em)),
                        }
                    ]
                },
                Rule {
                    selectors: vec![SimpleSelector::ClassSelector {
                        class_name: "rule".to_string(),
                    }],
                    declarations: vec![Declaration {
                        name: "backgound".to_string(),
                        value: CSSValue::Keyword("none".to_string())
                    }]
                },
            ]))
        );
    }

    #[test]
    fn test_whitespaces() {
        let input_1 = "  \t\n  \r\n ";
        let test_1 = String::from(input_1);
        let input_2 = "  \t\n 444 \r\n ";
        let test_2 = String::from(input_2);
        let mut parser = whitespaces();

        assert_eq!(parser.parse(input_1), Ok((test_1, "")));
        assert_eq!(
            parser.parse(input_2),
            Ok((String::from("  \t\n "), "444 \r\n "))
        );
    }

    #[test]
    fn test_rule() {
        let input_1 = ".test { display: flex; width: 1em }";
        assert_eq!(
            rule().parse(input_1),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::ClassSelector {
                        class_name: "test".to_string(),
                    }],
                    declarations: vec![
                        Declaration {
                            name: "display".to_string(),
                            value: CSSValue::Keyword("flex".to_string())
                        },
                        Declaration {
                            name: "width".to_string(),
                            value: CSSValue::Length((1, Unit::Em)),
                        }
                    ]
                },
                ""
            ))
        );
    }

    #[test]
    fn test_selectors() {
        let input = "a[display=flex], div[background~=g] {";
        assert_eq!(
            selectors().parse(input),
            Ok((
                vec![
                    SimpleSelector::AttributeSelector {
                        tag_name: "a".to_string(),
                        attribute: "display".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "flex".to_string()
                    },
                    SimpleSelector::AttributeSelector {
                        tag_name: "div".to_string(),
                        attribute: "background".to_string(),
                        op: AttributeSelectorOp::Contain,
                        value: "g".to_string()
                    }
                ],
                "{"
            ))
        );
    }

    #[test]
    fn test_declarations() {
        let input = "display: flex; width: 16em; }";
        assert_eq!(
            declarations().parse(input),
            Ok((
                vec![
                    Declaration {
                        name: "display".to_string(),
                        value: CSSValue::Keyword("flex".to_string())
                    },
                    Declaration {
                        name: "width".to_string(),
                        value: CSSValue::Length((16, Unit::Em))
                    }
                ],
                "}"
            ))
        );
    }

    #[test]
    fn test_declaratione() {
        let input_1 = "display: flex";
        let mut parser = declaration();

        assert_eq!(
            parser.parse(input_1),
            Ok((
                Declaration {
                    name: "display".to_string(),
                    value: CSSValue::Keyword("flex".to_string()),
                },
                ""
            ))
        );
    }

    #[test]
    fn test_css_value() {
        let expected = css_value().parse("1em");
        assert_eq!(expected, Ok((CSSValue::Length((1, Unit::Em)), "")))
    }
}
