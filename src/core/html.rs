use super::dom::{
    document::Document,
    element::{AttrMap, Element},
    node::Node,
    text::Text,
};
use super::fetch::Response;
#[allow(unused_imports)]
use combine::EasyParser;
use combine::{
    attempt,
    error::{StreamError, StringStreamError},
    many,
    parser::char::{newline, space},
};
use combine::{between, many1, parser, sep_by, Parser, Stream};
use combine::{choice, error::ParseError};
use combine::{
    parser::char::{char, letter},
    satisfy,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HTMLParseError {
    #[error("failed to parse; {0}")]
    InvalidResourceError(StringStreamError),
}

pub fn parse(response: Response) -> Result<Document, HTMLParseError> {
    let nodes = parse_without_normalziation(response.data);
    match nodes {
        Ok(nodes) => {
            let document_element = if nodes.len() == 1 {
                nodes.into_iter().nth(0).unwrap()
            } else {
                Element::new("html".to_string(), AttrMap::new(), nodes)
            };
            Ok(Document::new(
                response.url.to_string(),
                response.url.to_string(),
                document_element,
            ))
        }
        Err(e) => Err(e),
    }
}

// data -> nodes
pub fn parse_without_normalziation(data: Vec<u8>) -> Result<Vec<Box<Node>>, HTMLParseError> {
    let body = String::from_utf8(data).unwrap();

    nodes()
        .parse(&body as &str)
        .map(|(nodes, _)| nodes)
        .map_err(|e| HTMLParseError::InvalidResourceError(e))
}

/// <tag_name attr_name="attr_value" ...>(children)</tag_name>
fn element<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (open_tag(), nodes(), close_tag()).and_then(
        |((open_tag_name, attributes), children, close_tag_name)| {
            if open_tag_name == close_tag_name {
                Ok(Element::new(open_tag_name, attributes, children))
            } else {
                Err(<Input::Error as combine::error::ParseError<
                    char,
                    Input::Range,
                    Input::Position,
                >>::StreamError::message_static_message(
                    "tag name of open tag and close tag mismatched",
                ))
            }
        },
    )
}

fn nodes_<Input>() -> impl Parser<Input, Output = Vec<Box<Node>>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(many(choice((attempt(element()), attempt(text())))))
}

fn text<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c: char| c != '<')).map(|t| Text::new(t))
}

// <tag_name attr_name="attr_value">
fn open_tag<Input>() -> impl Parser<Input, Output = (String, AttrMap)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let open_tag_name = many1::<String, _, _>(letter());
    let open_tag_content = (
        open_tag_name,
        many::<String, _, _>(space().or(newline())),
        attributes(),
    )
        .map(|v: (String, _, AttrMap)| (v.0, v.2));
    between(char('<'), char('>'), open_tag_content)
}

// </tag_name>
fn close_tag<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let close_tag_name = many1::<String, _, _>(letter());
    let close_tag_content = (char('/'), close_tag_name).map(|v| v.1);
    between(char('<'), char('>'), close_tag_content)
}

// 解析所有属性
fn attributes<Input>() -> impl Parser<Input, Output = AttrMap>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by::<Vec<(String, String)>, _, _, _>(
        attribute(),
        many::<String, _, _>(space().or(newline())),
    )
    .map(|attrs: Vec<(String, String)>| {
        let m: AttrMap = attrs.into_iter().collect();
        m
    })
}

//  name1="value1" name2="value2"
fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let attribute_name = many1::<String, _, _>(letter());
    let attribute_inner_value =
        many1::<String, _, _>(satisfy(|c: char| c != '"')).map(|x| x.replace("&quot;", "\""));
    let attribute_value = between(char('"'), char('"'), attribute_inner_value);
    (
        attribute_name,
        many::<String, _, _>(space().or(newline())),
        char('='),
        many::<String, _, _>(space().or(newline())),
        attribute_value,
    )
        .map(|v| (v.0, v.4))
}

parser! {
    fn nodes[Input]()(Input) -> Vec<Box<Node>>
    where [Input: Stream<Token = char>]
    {
        nodes_()
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;
    use crate::core::dom::{
        document::Document,
        element::{AttrMap, Element},
        node::Node,
        text::Text,
    };

    #[test]
    fn test_parse_element() {
        assert_eq!(
            element().easy_parse("<p></p>"),
            Ok((Element::new("p".to_string(), AttrMap::new(), vec![]), ""))
        );

        assert_eq!(
            element().easy_parse("<p>Hello World</p>"),
            Ok((
                Element::new(
                    "p".to_string(),
                    AttrMap::new(),
                    vec![Text::new("Hello World".to_string())]
                ),
                ""
            ))
        );

        assert!(element().easy_parse("<p>Hello World</div>").is_err());
    }

    #[test]
    fn test_parse_text() {
        {
            assert_eq!(
                text().easy_parse("Hello World"),
                Ok((Text::new("Hello World".to_string()), ""))
            );
        }
        {
            assert_eq!(
                text().easy_parse("Hello World<"),
                Ok((Text::new("Hello World".to_string()), "<"))
            );
        }
    }

    #[test]
    fn test_parse_open_tag() {
        {
            assert_eq!(
                open_tag().easy_parse("<p>"),
                Ok((("p".to_string(), AttrMap::new()), ""))
            );
        }
        {
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            assert_eq!(
                open_tag().easy_parse("<p id=\"test\">"),
                Ok((("p".to_string(), attributes), ""))
            )
        }

        {
            let result = open_tag().easy_parse("<p id=\"test\" class=\"sample\">");
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            attributes.insert("class".to_string(), "sample".to_string());
            assert_eq!(result, Ok((("p".to_string(), attributes), "")));
        }
    }

    #[test]
    fn test_parse_close_tag() {
        let result = close_tag().easy_parse("</p>");
        assert_eq!(result, Ok(("p".to_string(), "")))
    }

    #[test]
    fn test_parse_attributes() {
        let mut expected_map = AttrMap::new();
        expected_map.insert("test".to_string(), "foobar".to_string());
        assert_eq!(
            attributes().easy_parse("test=\"foobar\""),
            Ok((expected_map, ""))
        );
    }

    #[test]
    fn test_parse_attribute() {
        assert_eq!(
            attribute().easy_parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );

        assert_eq!(
            attribute().easy_parse("test = \"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );

        assert_eq!(
            attribute().easy_parse("test = \"&quot;&quot;\""),
            Ok((("test".to_string(), "\"\"".to_string()), ""))
        )
    }
}
