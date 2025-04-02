use std::collections::HashMap;
use from_pest::FromPest;
use pest::Parser;
use crate::genders::ast::GendersFile;
use crate::genders::parser::{GendersParser, Rule};

#[derive(Debug)]
pub enum AttributeValue<'src> {
    Boolean(bool),
    Number(u64),
    String(&'src str),
}

#[derive(Debug)]
pub struct Genders<'src> {
    hosts: HashMap<&'src str, HashMap<&'src str, Option<AttributeValue<'src>>>>
}

impl <'src> TryFrom<&'src str> for Genders<'src> {
    type Error = anyhow::Error;

    fn try_from(file_contents: &'src str) -> Result<Self, Self::Error> {
        let mut pairs = GendersParser::parse(Rule::genders_file, file_contents)?;
        let ast = GendersFile::from_pest(&mut pairs)?;
        Ok(ast.try_into()?)
    }
}

mod parser {
    #[derive(pest_derive::Parser)]
    #[grammar = "genders.pest"]
    pub(super) struct GendersParser;
}

mod ast {
    use anyhow::anyhow;
    use std::collections::HashMap;
    use super::{AttributeValue, Genders, Rule};
    use crate::util::span_into_str;
    use from_pest::{ConversionError, FromPest};
    use pest::iterators::Pairs;
    use pest_ast::FromPest;

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::ident))]
    struct Ident<'src> {
        #[pest_ast(outer(with(span_into_str)))]
        value: &'src str,
    }

    #[derive(FromPest)]
    #[pest_ast(rule(Rule::attribute_value_bool))]
    struct AttributeValueBool {
        #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
        value: bool,
    }

    #[derive(FromPest)]
    #[pest_ast(rule(Rule::attribute_value_numeric))]
    struct AttributeValueNumber {
        #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
        value: u64,
    }

    #[derive(FromPest)]
    #[pest_ast(rule(Rule::attribute_value_string))]
    struct AttributeValueString<'src> {
        #[pest_ast(outer(with(span_into_str)))]
        value: &'src str,
    }

    #[derive(Debug)]
    struct Attribute<'src> {
        name: &'src str,
        value: Option<AttributeValue<'src>>,
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::attribute_list))]
    struct AttributeList<'src> {
        attributes: Vec<Attribute<'src>>,
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::host_entry))]
    struct HostEntry<'src> {
        name: Ident<'src>,
        attributes: AttributeList<'src>,
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::genders_file))]
    pub(super) struct GendersFile<'src> {
        hosts: Vec<HostEntry<'src>>,
        _eoi: Vec<EOI>,
    }

    #[derive(Debug, FromPest)]
    #[pest_ast(rule(Rule::EOI))]
    struct EOI;

    impl <'src> FromPest<'src> for AttributeValue<'src> {
        type Rule = Rule;
        type FatalError = from_pest::Void;

        fn from_pest(pest: &mut Pairs<'src, Self::Rule>) -> Result<Self, ConversionError<Self::FatalError>> {
            let pair = pest.next().ok_or(ConversionError::NoMatch)?;
            match pair.as_rule() {
                Rule::attribute_value => Self::from_pest(&mut pair.into_inner()),
                Rule::attribute_value_bool => Ok(AttributeValue::Boolean(AttributeValueBool::from_pest(&mut Pairs::single(pair))?.value)),
                Rule::attribute_value_numeric => Ok(AttributeValue::Number(AttributeValueNumber::from_pest(&mut Pairs::single(pair))?.value)),
                Rule::attribute_value_string => Ok(AttributeValue::String(AttributeValueString::from_pest(&mut Pairs::single(pair))?.value)),
                r => {
                    println!("Bad rule: {r:?}");
                    Err(ConversionError::NoMatch)
                },
            }
        }
    }

    impl <'src> FromPest<'src> for Attribute<'src> {
        type Rule = Rule;
        type FatalError = from_pest::Void;

        fn from_pest(pest: &mut Pairs<'src, Self::Rule>) -> Result<Self, ConversionError<Self::FatalError>> {
            let pair = pest.next().ok_or(ConversionError::NoMatch)?;
            if pair.as_rule() != Rule::attribute {
                return Err(ConversionError::NoMatch);
            }

            let mut pair = pair.into_inner();
            let name = pair.next().ok_or(ConversionError::NoMatch)?;

            Ok(Self {
                name: name.as_span().as_str(),
                value: pair.next().map(|p| AttributeValue::from_pest(&mut Pairs::single(p))).transpose()?,
            })
        }
    }

    impl <'src> TryFrom<GendersFile<'src>> for Genders<'src> {
        type Error = anyhow::Error;

        fn try_from(genders: GendersFile<'src>) -> Result<Self, Self::Error> {
            let mut hosts = HashMap::new();
            for host in genders.hosts {
                let mut attributes_map = HashMap::new();
                for attribute in host.attributes.attributes {
                    if attributes_map.insert(attribute.name, attribute.value).is_some() {
                        return Err(anyhow!("Duplicate attribute {} on host {}", attribute.name, host.name.value));
                    }
                }
                if hosts.insert(host.name.value, attributes_map).is_some() {
                    return Err(anyhow!("Duplicate host {}", host.name.value));
                }
            }
            Ok(Self { hosts })
        }
    }
}
