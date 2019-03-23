#![allow(dead_code)]
//! This module contains all css theming releated resources.

use cssparser::CowRcStr;
use cssparser::{
    self, DeclarationListParser, ParseError, Parser, ParserInput, SourceLocation, Token,
};

use crate::color::Color;

use std::ops::Add;
use std::path::Path;
use std::sync::Arc;

use crate::style_system::{KuchikiParser, Selector, Selectors};
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

use stretch::{
    geometry::Rect,
    style::*
};

#[derive(Debug, Default, Clone)]
pub struct Theme {
    parent: Option<Arc<Theme>>,
    rules: Vec<Rule>,
}

impl Theme {
    pub fn new() -> Self {
        Theme::parse("")
    }

    pub fn parse(s: &str) -> Self {
        Theme {
            parent: None,
            rules: parse(s).0,
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Theme, String> {
        let file = (File::open(path).map_err(|err| format!("failed to open css: {}", err)))?;
        let mut reader = BufReader::new(file);
        let mut css = String::new();
        let res = reader
            .read_to_string(&mut css)
            .map_err(|err| format!("failed to read css: {}", err));
        match res {
            Ok(_) => Ok(Theme::parse(&css)),
            Err(err) => Err(err),
        }
    }

    pub fn all_rules(&self) -> Vec<Rule> {
        if let Some(ref parent) = self.parent {
            self.rules
                .iter()
                .chain(parent.rules.iter())
                .cloned()
                .collect()
        } else {
            self.rules.clone()
        }
    }

    // pub fn get(&self, property: &str, query: &Selector) -> Option<Value> {
    //     let mut matches: Vec<(bool, Specificity, Value)> = Vec::new();

    //     for rule in self.all_rules().iter().rev() {
    //         let matching_selectors = rule
    //             .selectors
    //             .iter()
    //             .filter(|x| x.matches(query))
    //             .collect::<Vec<_>>();

    //         if !matching_selectors.is_empty() {
    //             if let Some(decl) = rule
    //                 .declarations
    //                 .iter()
    //                 .find(|decl| decl.property == property)
    //             {
    //                 let highest_specifity = matching_selectors
    //                     .iter()
    //                     .map(|sel| sel.specificity())
    //                     .max()
    //                     .unwrap();
    //                 matches.push((decl.important, highest_specifity, decl.value.clone()));
    //             }
    //         }
    //     }

    //     matches.sort_by_key(|x| (x.0, x.1));
    //     matches.last().map(|x| x.2.clone())
    // }

    // pub fn color(&self, property: &str, query: &Selector) -> Color {
    //     let default = Color { data: 0 };
    //     self.get(property, query)
    //         .map(|v| v.color().unwrap_or(default))
    //         .unwrap_or(default)
    // }

    // pub fn uint(&self, property: &str, query: &Selector) -> u32 {
    //     self.get(property, query)
    //         .map(|v| v.uint().unwrap_or(0))
    //         .unwrap_or(0)
    // }

    // pub fn float(&self, property: &str, query: &Selector) -> f32 {
    //     self.get(property, query)
    //         .map(|v| v.float().unwrap_or(1.0))
    //         .unwrap_or(1.0)
    // }

    // pub fn string(&self, property: &str, query: &Selector) -> String {
    //     self.get(property, query)
    //         .map(|v| v.string().unwrap_or(String::default()))
    //         .unwrap_or(String::default())
    // }
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub selectors: Selectors,
    pub declarations: Vec<Declaration>,
}

#[derive(Clone, Debug)]
pub enum SelectorRelation {
    Ancestor(Selector),
    Parent(Selector),
}

impl<T: Into<String>> From<T> for Selector {
    fn from(_t: T) -> Self {
        unreachable!("asdasdasd")
        // Selector::new().with(t.into())
    }
}

/// Describes the specificity of a selector.
///
/// The indexes are as follows:
/// 0 - number of IDs (most important)
/// 1 - number of classes and pseudo-classes
/// 2 - number of elements (least important)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Specificity([u8; 3]);

impl Add<Self> for Specificity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Specificity([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}

// #[derive(Clone, Debug, Default)]
// pub struct Selector {
//     pub element: Option<String>,
//     pub classes: HashSet<String>,
//     pub pseudo_classes: HashSet<String>,
//     pub relation: Option<Box<SelectorRelation>>,
// }

// impl Selector {
//     pub fn new() -> Self {
//         Selector {
//             element: None,
//             classes: HashSet::new(),
//             pseudo_classes: HashSet::new(),
//             relation: None,
//         }
//     }

//     fn specificity(&self) -> Specificity {
//         let s = Specificity([
//             0,
//             (self.classes.len() + self.pseudo_classes.len()) as u8,
//             if self.element.is_some() { 1 } else { 0 },
//         ]);

//         if let Some(ref relation) = self.relation {
//             match **relation {
//                 SelectorRelation::Ancestor(ref x) | SelectorRelation::Parent(ref x) => {
//                     return x.specificity() + s;
//                 }
//             }
//         }

//         s
//     }

//     pub fn matches(&self, other: &Selector) -> bool {
//         if self.element.is_some() && self.element != other.element {
//             return false;
//         }

//         if !other.classes.is_superset(&self.classes) {
//             return false;
//         }

//         if !other.pseudo_classes.is_superset(&self.pseudo_classes) {
//             return false;
//         }

//         true
//     }

//     pub fn with_class<S: Into<String>>(mut self, class: S) -> Self {
//         self.classes.insert(class.into());
//         self
//     }

//     pub fn with<S: Into<String>>(mut self, element: S) -> Self {
//         self.element = Some(element.into());
//         self
//     }

//     pub fn without_class<S: Into<String>>(mut self, class: S) -> Self {
//         self.classes.remove(&class.into());
//         self
//     }

//     pub fn with_pseudo_class<S: Into<String>>(mut self, pseudo_class: S) -> Self {
//         self.pseudo_classes.insert(pseudo_class.into());
//         self
//     }

//     pub fn without_pseudo_class<S: Into<String>>(mut self, pseudo_class: S) -> Self {
//         self.pseudo_classes.remove(&pseudo_class.into());
//         self
//     }
// }

// impl Selector {
//     pub fn is_empty(&self) -> bool {
//         self.element.is_none() && self.classes.is_empty() && self.pseudo_classes.is_empty()
//     }
// }

#[derive(Clone, Debug)]
pub struct Declaration {
    pub property: String,
    pub value: Value,
    pub important: bool,
}

#[derive(Clone, Debug)]
pub enum Value {
    UInt(u32),
    Float(f32),
    Color(Color),
    Str(String),
    Ident(String),
    Rect(Rect<Dimension>),
}

impl Value {
    pub fn uint(&self) -> Option<u32> {
        match *self {
            Value::UInt(x) => Some(x),
            Value::Float(x) => Some(x as u32),
            _ => None,
        }
    }

    pub fn float(&self) -> Option<f32> {
        match *self {
            Value::Float(x) => Some(x),
            Value::UInt(x) => Some(x as f32),
            _ => None,
        }
    }

    pub fn color(&self) -> Option<Color> {
        match *self {
            Value::Color(x) => Some(x),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<String> {
        match self {
            Value::Str(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn rect_dimension(&self) -> Option<Rect<Dimension>> {
        match self {
            Value::Rect(r) => Some(r.clone()),
            _ => None,
        }
    }
    pub fn dimension(&self) -> Option<Dimension> {
        match self.ident() {
            Some("auto") => return Some(Dimension::Auto),
            Some("undefined") | Some("none") => return Some(Dimension::Undefined),
            _ => (),
        };
        match self {
            Value::Float(x) => Some(Dimension::Points(*x)),
            Value::UInt(x) => Some(Dimension::Points(*x as f32)),
            _ => None,
        }
    }

    fn ident(&self) -> Option<&str> {
        match self {
            Value::Ident(ref ident) => Some(ident),
            _ => None,
        }
    }

    pub fn display(&self) -> Option<Display> {
        match self.ident() {
            Some("none") => Some(Display::None),
            Some("flex") => Some(Display::Flex),
            _ => None,
        }
    }

    pub fn position_type(&self) -> Option<PositionType> {
        match self.ident() {
            Some("absolute") => Some(PositionType::Absolute),
            Some("relative") => Some(PositionType::Relative),
            _ => None,
        }
    }
    pub fn direction(&self) -> Option<Direction> {
        match self.ident() {
            Some("inherit") => Some(Direction::Inherit),
            Some("rtl") => Some(Direction::RTL),
            Some("ltr") => Some(Direction::LTR),
            _ => None,
        }
    }
    pub fn flex_direction(&self) -> Option<FlexDirection> {
        match self.ident() {
            Some("column") => Some(FlexDirection::Column),
            Some("row") => Some(FlexDirection::Row),
            Some("columnreverse") => Some(FlexDirection::ColumnReverse),
            Some("rowreverse") => Some(FlexDirection::RowReverse),
            _ => None,
        }
    }
    pub fn flex_wrap(&self) -> Option<FlexWrap> {
        match self.ident() {
            Some("noWrap") => Some(FlexWrap::NoWrap),
            Some("wrap") => Some(FlexWrap::Wrap),
            Some("wrapReverse") => Some(FlexWrap::WrapReverse),
            _ => None,
        }
    }
    pub fn overflow(&self) -> Option<Overflow> {
        match self.ident() {
            Some("hidden") => Some(Overflow::Hidden),
            Some("scroll") => Some(Overflow::Scroll),
            Some("visible") => Some(Overflow::Visible),
            _ => None,
        }
    }
    pub fn align_items(&self) -> Option<AlignItems> {
        match self.ident() {
            Some("baseline") => Some(AlignItems::Baseline),
            Some("center") => Some(AlignItems::Center),
            Some("flexend") => Some(AlignItems::FlexEnd),
            Some("flexstart") => Some(AlignItems::FlexStart),
            Some("stretch") => Some(AlignItems::Stretch),
            _ => None,
        }
    }
    pub fn align_self(&self) -> Option<AlignSelf> {
        match self.ident() {
            Some("auto") => Some(AlignSelf::Auto),
            Some("baseline") => Some(AlignSelf::Baseline),
            Some("center") => Some(AlignSelf::Center),
            Some("flexend") => Some(AlignSelf::FlexEnd),
            Some("flexstart") => Some(AlignSelf::FlexStart),
            Some("stretch") => Some(AlignSelf::Stretch),
            _ => None,
        }
    }
    pub fn align_content(&self) -> Option<AlignContent> {
        match self.ident() {
            Some("center") => Some(AlignContent::Center),
            Some("flexend") => Some(AlignContent::FlexEnd),
            Some("flexstart") => Some(AlignContent::FlexStart),
            Some("spacearound") => Some(AlignContent::SpaceAround),
            Some("spacebetween") => Some(AlignContent::SpaceBetween),
            Some("stretch") => Some(AlignContent::Stretch),
            _ => None,
        }
    }
    pub fn justify_content(&self) -> Option<JustifyContent> {
        match self.ident() {
            Some("center") => Some(JustifyContent::Center),
            Some("flexend") => Some(JustifyContent::FlexEnd),
            Some("flexstart") => Some(JustifyContent::FlexStart),
            Some("spacearound") => Some(JustifyContent::SpaceAround),
            Some("spacebetween") => Some(JustifyContent::SpaceBetween),
            Some("spaceevenly") => Some(JustifyContent::SpaceEvenly),
            _ => None,
        }
    }
    // pub fn position(&self) -> Option<Rect<Dimension>> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn margin(&self) -> Option<Rect<Dimension>> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn padding(&self) -> Option<Rect<Dimension>> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn border(&self) -> Option<Rect<Dimension>> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn flex_grow(&self) -> Option<f32> {
    //     match self.ident() {
    //         Some("") => Some(f32::),
    //         _ => None,
    //     }
    // }
    // pub fn flex_shrink(&self) -> Option<f32> {
    //     match self.ident() {
    //         Some("") => Some(f32::),
    //         _ => None,
    //     }
    // }
    // pub fn flex_basis(&self) -> Option<Dimension> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn size(&self) -> Option<Size<Dimension>> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn min_size(&self) -> Option<Size<Dimension>> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn max_size(&self) -> Option<Size<Dimension>> {
    //     match self.ident() {
    //         Some("") => Some(Dimension::),
    //         _ => None,
    //     }
    // }
    // pub fn aspect_ratio(&self) -> Option<Number> {
    //     match self.ident() {
    //         Some("") => Some(Number::),
    //         _ => None,
    //     }
    // }
}

#[derive(Clone, Debug)]
pub enum CustomParseError {
    InvalidColorName(String),
    InvalidColorHex(String),
    InvalidStringName(String),
}

impl<'t> From<CustomParseError> for ParseError<'t, CustomParseError> {
    fn from(_e: CustomParseError) -> Self {
        panic!("wtf");
    }
}

struct RuleParser;

impl RuleParser {
    fn new() -> Self {
        RuleParser {}
    }
}

impl<'i> cssparser::QualifiedRuleParser<'i> for RuleParser {
    type Prelude = Selectors;
    type QualifiedRule = Rule;
    type Error = CustomParseError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let res = parse_selectors(input)?;
        Ok(res)
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        _location: SourceLocation,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let decl_parser = DeclarationParser {};

        let decls = DeclarationListParser::new(input, decl_parser).collect::<Vec<_>>();

        for decl in &decls {
            match *decl {
                Ok(_) => {}
                Err(ref e) => {
                    eprintln!("{:?}", e);
                    // match e.0 {
                    //     ParseError::Basic(ref e) => ,
                    //     ParseError::Custom(ref e) => eprintln!("{:?}", e),
                    // }
                    println!("Error occured in `{}`", e.1); // input.slice(e.span.clone()));
                }
            }
        }

        let decls = decls.into_iter().filter_map(|decl| decl.ok()).collect();

        Ok(Rule {
            selectors: selectors,
            declarations: decls,
        })
    }
}

impl<'i> cssparser::AtRuleParser<'i> for RuleParser {
    /// The intermediate representation of prelude of an at-rule without block;
    type PreludeNoBlock = ();

    /// The intermediate representation of prelude of an at-rule with block;
    type PreludeBlock = ();
    type AtRule = Rule;
    type Error = CustomParseError;
}

fn parse_selectors<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Selectors, ParseError<'i, CustomParseError>> {
    use selectors::parser::SelectorList;
    match SelectorList::parse(&KuchikiParser, input) {
        Ok(list) => Ok(Selectors(list.0.into_iter().map(Selector).collect())),
        _ => unreachable!("asd"),
    }
}

struct DeclarationParser;

impl DeclarationParser {
    fn parse_dimension<'i, 't>(
        &mut self,
        input: &mut Parser<'i, 't>
    ) -> Result<Dimension, ParseError<'i, CustomParseError>> {
        let location = input.current_source_location();
        match *input.next()? {
            Token::Number { value, .. } => Ok(Dimension::Points(value)),
            Token::Percentage { unit_value, .. } => Ok(Dimension::Percent(unit_value)),
            Token::Ident(ref id) if id == &"auto" => Ok(Dimension::Auto),
            ref t => return Err(location.new_unexpected_token_error(t.clone()))
        }
    }
}

impl<'i> cssparser::DeclarationParser<'i> for DeclarationParser {
    type Declaration = Declaration;
    type Error = CustomParseError;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, ParseError<'i, Self::Error>> {
        let value = match &*name {
            "color" | "border-color" | "icon-color" => Value::Color(parse_basic_color(input)?),

            "background" | "foreground" => Value::Color(parse_basic_color(input)?),

            "font-family" | "icon-font-family" => Value::Str(parse_string(input)?),

            "margin" | "padding" => {
                let mut array: [_; 4] = [None; 4];
                let mut i: usize = 0;
                while let Ok(n) = self.parse_dimension(input) {
                    array[i] = Some(n);
                    i += 1;
                }

                match i {
                    4 /*tlrb*/ => Value::Rect(Rect { top: array[0].unwrap(), start: array[1].unwrap(), end: array[2].unwrap(), bottom: array[3].unwrap() }),
                    3 /*thb*/ => Value::Rect(Rect { top: array[0].unwrap(), start: array[1].unwrap(), end: array[1].unwrap(), bottom: array[2].unwrap() }),
                    2 /*vh*/ => Value::Rect(Rect { top: array[0].unwrap(), start: array[1].unwrap(), end: array[1].unwrap(), bottom: array[0].unwrap() }),
                    1 /*vh*/ => Value::Rect(Rect { top: array[0].unwrap(), start: array[0].unwrap(), end: array[0].unwrap(), bottom: array[0].unwrap() }),
                    _ => unimplemented!(),
                }
            }

            "border-radius" | "border-width" | "width" | "height" | "min-width" | "min-height"
            | "max-width" | "max-height" | "padding-top" | "padding-right" | "padding-bottom"
            | "padding-left" | "margin-top" | "margin-right" | "margin-bottom"
            | "margin-left" | "font-size" | "icon-size" | "icon-margin" => {
                match input.next()?.clone() {
                    Token::Number {
                        int_value: Some(x),
                        has_sign,
                        ..
                    } if !has_sign && x >= 0 => Value::UInt(x as u32),
                    t => {
                        return Err(input
                            .current_source_location()
                            .new_basic_unexpected_token_error(t.clone())
                            .into());
                    }
                }
            }

            "opacity" => match input.next()?.clone() {
                Token::Number { value: x, .. } => Value::Float(x as f32),
                t => {
                    return Err(input
                        .current_source_location()
                        .new_basic_unexpected_token_error(t.clone())
                        .into());
                }
            },

            _ => match input.next()?.clone() {
                Token::Ident(rc) => {
                    println!("ident {}: {:?}", name, rc);
                    Value::Ident(rc.to_string().to_lowercase().to_string())
                }
                _ => {
                    return Err(input
                        .current_source_location()
                        .new_basic_unexpected_token_error(input.next()?.clone())
                        .into());
                }
            },
        };

        Ok(Declaration {
            property: name.to_string(),
            value: value,
            important: input.r#try(cssparser::parse_important).is_ok(),
        })
    }
}

impl<'i> cssparser::AtRuleParser<'i> for DeclarationParser {
    type PreludeBlock = ();
    type PreludeNoBlock = ();
    type AtRule = Declaration;
    type Error = CustomParseError;
}

fn css_color(name: &str) -> Option<Color> {
    Some(hex(match name {
        "transparent" => return Some(Color { data: 0 }),

        "black" => 0x000_000,
        "silver" => 0xc0c_0c0,
        "gray" | "grey" => 0x808_080,
        "white" => 0xfff_fff,
        "maroon" => 0x800_000,
        "red" => 0xff0_000,
        "purple" => 0x800_080,
        "fuchsia" => 0xff0_0ff,
        "green" => 0x008_000,
        "lime" => 0x00f_f00,
        "olive" => 0x808_000,
        "yellow" => 0xfff_f00,
        "navy" => 0x000_080,
        "blue" => 0x000_0ff,
        "teal" => 0x008_080,
        "aqua" => 0x00f_fff,
        _ => return None,
    }))
}

fn css_string(name: &str) -> Option<String> {
    Some(String::from(name))
}

fn parse_string<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<String, ParseError<'i, CustomParseError>> {
    Ok(match input.next()?.clone() {
        Token::QuotedString(s) => match css_string(&s) {
            Some(string) => string,
            None => return Err(CustomParseError::InvalidStringName(s.to_string()).into()),
        },

        t => {
            let basic_error = input
                .current_source_location()
                .new_basic_unexpected_token_error(t.clone());
            return Err(basic_error.into());
        }
    })
}

fn parse_basic_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Color, ParseError<'i, CustomParseError>> {
    Ok(match input.next()?.clone() {
        Token::Ident(s) => match css_color(&s) {
            Some(color) => color,
            None => return Err(CustomParseError::InvalidColorName(s.to_string()).into()),
        },

        Token::IDHash(hash) | Token::Hash(hash) => match hash.len() {
            6 | 8 => {
                let mut x = match u32::from_str_radix(&hash, 16) {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(CustomParseError::InvalidColorHex(hash.to_string()).into());
                    }
                };

                if hash.len() == 6 {
                    x = (x << 8) | 0xFF;
                }

                Color { data: x }
            }
            _ => return Err(CustomParseError::InvalidColorHex(hash.to_string()).into()),
        },

        t => {
            let basic_error = input
                .current_source_location()
                .new_basic_unexpected_token_error(t.clone());
            return Err(basic_error.into());
        }
    })
}

use crate::manager::*;

#[derive(Debug)]
pub struct Stylesheet(pub Vec<Rule>);

// impl Stylesheet {
//      pub fn get(&self, property: &str) -> Option<Value> {
//         let mut matches: Vec<(bool, Specificity, Value)> = Vec::new();

//         for rule in self.all_rules().iter().rev() {
//             let matching_selectors = rule
//                 .selectors
//                 .iter()
//                 .filter(|x| x.matches(query))
//                 .collect::<Vec<_>>();

//             if !matching_selectors.is_empty() {
//                 if let Some(decl) = rule
//                     .declarations
//                     .iter()
//                     .find(|decl| decl.property == property)
//                 {
//                     let highest_specifity = matching_selectors
//                         .iter()
//                         .map(|sel| sel.specificity())
//                         .max()
//                         .unwrap();
//                     matches.push((decl.important, highest_specifity, decl.value.clone()));
//                 }
//             }
//         }

//         matches.sort_by_key(|x| (x.0, x.1));
//         matches.last().map(|x| x.2.clone())
//     }
// }

impl Load<Ctx, SimpleKey> for Stylesheet {
    type Error = Error;

    fn load(
        key: SimpleKey,
        _storage: &mut Storage<Ctx, SimpleKey>,
        _ctx: &mut Ctx,
    ) -> Result<Loaded<Self, SimpleKey>, Error> {
        match key {
            SimpleKey::Path(path) => {
                println!("Load Stylesheet {}", path.display());
                let mut fh = File::open(path).map_err(Error::IOError)?;
                let mut buf = String::new();
                fh.read_to_string(&mut buf).map_err(Error::IOError)?;

                let stylesheet = parse(&buf);
                // storage.get::<ShaderSet>(&dep, ctx).unwrap();
                Ok(Loaded::without_dep(stylesheet))
            }

            SimpleKey::Logical(_) => Err(Error::CannotLoadFromLogical),
        }
    }
}

pub fn parse(s: &str) -> Stylesheet {
    let mut input = ParserInput::new(s);
    let mut parser = Parser::new(&mut input);
    let rule_parser = RuleParser::new();

    let rules = {
        let rule_list_parser =
            cssparser::RuleListParser::new_for_stylesheet(&mut parser, rule_parser);
        rule_list_parser.collect::<Vec<_>>()
    };

    for rule in &rules {
        match *rule {
            Ok(_) => {}
            Err(ref e) => {
                eprintln!("{:?}", e);
                // println!("Error occured in `{}`", parser.slice(e.span.clone()));
            }
        }
    }

    Stylesheet(rules.into_iter().filter_map(|rule| rule.ok()).collect())
}

const fn hex(data: u32) -> Color {
    Color {
        data: 0xFF | (data << 8),
    }
}
