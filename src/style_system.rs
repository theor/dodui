use specs::prelude::*;
use selectors::*;
use selectors::parser::*;

use selectors::parser::{SelectorImpl, Parser, SelectorList, Selector as GenericSelector, NonTSPseudoClass};
use selectors::context::*;
use selectors::matching::ElementSelectorFlags;
use selectors::attr::{AttrSelectorOperation, NamespaceConstraint, CaseSensitivity};
use cssparser::{self, ToCss, CowRcStr, SourceLocation, ParseError};

// use string_interner::StringInterner;
// use std::sync::Mutex;

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct Sym(string_interner::Sym);

// impl Sym {
//     pub fn resolve<'a>(&'a self) -> Option<String> {
//         let si: &string_interner::DefaultStringInterner = &STRING_INTERNER.lock().unwrap();
//         si.resolve(self.0).map(|x| x.to_owned())
//     }
// }

// lazy_static! {
//     static ref STRING_INTERNER: Mutex<string_interner::DefaultStringInterner> = {
//         let m = string_interner::DefaultStringInterner::new();
//         Mutex::new(m)
//     };
// }

// impl std::fmt::Display for Sym {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match STRING_INTERNER.lock().unwrap().resolve(self.0) {
//             Some(x) => { x.fmt(f); Ok(()) },
//             None => panic!("resolve"),
//         }
//     }
// }

// impl std::borrow::Borrow<Sym> for String {
//     fn borrow(&self) -> &Sym {
        
//         Sym(STRING_INTERNER.lock().unwrap().get_or_intern(self.as_str()))
//     }
// }

// impl<'a> std::convert::From<&'a str> for Sym {
//     fn from(s:&'a str) -> Self {
//         let sym = STRING_INTERNER.lock().unwrap().get_or_intern(s);
//         Self(sym)
//     }
// }

#[derive(Debug, Clone)]
pub struct KuchikiSelectors;

impl SelectorImpl for KuchikiSelectors {
    type AttrValue = String;
    type Identifier = String;//LocalName;
    type ClassName = String;//LocalName;
    type LocalName = String;//LocalName;
    type NamespacePrefix = String;//LocalName;
    type NamespaceUrl = String;//Namespace;
    type BorrowedNamespaceUrl = str;//Namespace;
    type BorrowedLocalName = String;//LocalName;

    type NonTSPseudoClass = PseudoClass;
    type PseudoElement = PseudoElement;

    type ExtraMatchingData = ();
}



#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum PseudoClass {
    AnyLink,
    Link,
    Visited,
    Active,
    Focus,
    Hover,
    Enabled,
    Disabled,
    Checked,
    Indeterminate,
}

impl NonTSPseudoClass for PseudoClass {
    type Impl = KuchikiSelectors;

    fn is_active_or_hover(&self) -> bool {
        matches!(*self, PseudoClass::Active | PseudoClass::Hover)
    }
}

impl ToCss for PseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result where W: std::fmt::Write {
        dest.write_str(match *self {
            PseudoClass::AnyLink => ":any-link",
            PseudoClass::Link => ":link",
            PseudoClass::Visited => ":visited",
            PseudoClass::Active => ":active",
            PseudoClass::Focus => ":focus",
            PseudoClass::Hover => ":hover",
            PseudoClass::Enabled => ":enabled",
            PseudoClass::Disabled => ":disabled",
            PseudoClass::Checked => ":checked",
            PseudoClass::Indeterminate => ":indeterminate",
        })
    }
}



#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum PseudoElement {}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> std::fmt::Result where W: std::fmt::Write {
        match *self {
        }
    }
}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = KuchikiSelectors;
}

pub struct KuchikiParser;

impl<'i> Parser<'i> for KuchikiParser {
    type Impl = KuchikiSelectors;
    type Error = SelectorParseErrorKind<'i>;

    fn parse_non_ts_pseudo_class(&self, location: SourceLocation, name: CowRcStr<'i>)
                                 -> Result<PseudoClass, ParseError<'i, SelectorParseErrorKind<'i>>> {
        use self::PseudoClass::*;
             if name.eq_ignore_ascii_case("any-link") { Ok(AnyLink) }
        else if name.eq_ignore_ascii_case("link") { Ok(Link) }
        else if name.eq_ignore_ascii_case("visited") { Ok(Visited) }
        else if name.eq_ignore_ascii_case("active") { Ok(Active) }
        else if name.eq_ignore_ascii_case("focus") { Ok(Focus) }
        else if name.eq_ignore_ascii_case("hover") { Ok(Hover) }
        else if name.eq_ignore_ascii_case("enabled") { Ok(Enabled) }
        else if name.eq_ignore_ascii_case("disabled") { Ok(Disabled) }
        else if name.eq_ignore_ascii_case("checked") { Ok(Checked) }
        else if name.eq_ignore_ascii_case("indeterminate") { Ok(Indeterminate) }
        else {
            Err(location.new_custom_error(
                SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name)
            ))
        }
    }
}

/// A pre-compiled list of CSS Selectors.
#[derive(Clone)]
pub struct Selectors(pub Vec<Selector>);

/// A pre-compiled CSS Selector.
#[derive(Clone)]
pub struct Selector(pub GenericSelector<KuchikiSelectors>);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Specificity(u32);

impl Selectors {
    /// Compile a list of selectors. This may fail on syntax errors or unsupported selectors.
    // #[inline]
    pub fn compile(s: &str) -> Result<Selectors, ()> {
        let mut input = cssparser::ParserInput::new(s);
        match SelectorList::parse(&KuchikiParser, &mut cssparser::Parser::new(&mut input)) {
            Ok(list) => Ok(Selectors(list.0.into_iter().map(Selector).collect())),
            Err(_) => Err(()),
        }
    }

    // /// Returns whether the given element matches this list of selectors.
    // #[inline]
    pub fn matches(&self, element: &EntityElement) -> bool {
        self.0.iter().any(|s| s.matches(element))
    }

    // /// Filter an element iterator, yielding those matching this list of selectors.
    // #[inline]
    // pub fn filter<I>(&self, iter: I) -> Select<I, &Selectors>
    // where I: Iterator<Item=EntityElement> {
    //     Select {
    //         iter: iter,
    //         selectors: self,
    //     }
    // }
}
#[derive(Debug, Clone)]
pub struct EntityElement {
    id: Option<String>,
    typeid: String,
    classes: Vec<String>,
    pseudo: Pseudo,
}

impl specs::Component for EntityElement {
    type Storage = DenseVecStorage<Self>;
}

impl EntityElement {
    pub fn new(typeid: String) -> Self {
        EntityElement {
            typeid,
            id: None,
            pseudo: Pseudo { hover: false},
            classes: Vec::new(),
        }
    }

    pub fn with_hover(self, hover: bool) -> Self {
        EntityElement {
            pseudo: Pseudo { hover },
            ..self
        }
    }

    pub fn with_id(self, id: String) -> Self {
        EntityElement {
            id: Some(id),
            ..self
        }
    }

    pub fn add_class(self, cl: String) -> Self {
        EntityElement {
            classes: vec![cl],
            ..self
        }
    }
}

impl Element for EntityElement {
    type Impl = KuchikiSelectors;

    /// Converts self into an opaque representation.
    fn opaque(&self) -> OpaqueElement { OpaqueElement::new(&self) }

    // TODO
    fn parent_element(&self) -> Option<Self> { None }

    /// Whether the parent node of this element is a shadow root.
    fn parent_node_is_shadow_root(&self) -> bool { false }

    /// The host of the containing shadow root, if any.
    fn containing_shadow_host(&self) -> Option<Self> { None }

    // fn pseudo_element_originating_element(&self) -> Option<Self> {
    //     self.parent_element()
    // }

    /// Skips non-element nodes
    fn prev_sibling_element(&self) -> Option<Self> { None }

    /// Skips non-element nodes
    fn next_sibling_element(&self) -> Option<Self> { None }

    fn is_html_element_in_html_document(&self) -> bool { false }

    fn local_name(&self) -> &<Self::Impl as SelectorImpl>::BorrowedLocalName { 
        // &self.typeid.resolve().unwrap()
        &self.typeid
    }

    /// Empty string for no namespace
    fn namespace(&self) -> &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl { &"" }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        false
    }

    fn match_non_ts_pseudo_class<F>(
        &self,
        pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        context: &mut MatchingContext<Self::Impl>,
        flags_setter: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags) {
            match pc {
                PseudoClass::Hover => self.pseudo.hover,
                _ => false,
            }
        }

    fn match_pseudo_element(
        &self,
        pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        context: &mut MatchingContext<Self::Impl>,
    ) -> bool { false }

    /// Whether this element is a `link`.
    fn is_link(&self) -> bool { false }

    /// Returns whether the element is an HTML <slot> element.
    fn is_html_slot_element(&self) -> bool { false }

    /// Returns the assigned <slot> element this element is assigned to.
    ///
    /// Necessary for the `::slotted` pseudo-class.
    // fn assigned_slot(&self) -> Option<Self> {
    //     None
    // }

    fn has_id(
        &self,
        id: &<Self::Impl as SelectorImpl>::Identifier,
        case_sensitivity: CaseSensitivity,
    ) -> bool { self.id.as_ref().map_or(false, |x| x == id) }

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::ClassName,
        case_sensitivity: CaseSensitivity,
    ) -> bool { self.classes.contains(name) }

    /// Returns whether this element matches `:empty`.
    ///
    /// That is, whether it does not contain any child element or any non-zero-length text node.
    /// See http://dev.w3.org/csswg/selectors-3/#empty-pseudo
    fn is_empty(&self) -> bool { false }

    /// Returns whether this element matches `:root`,
    /// i.e. whether it is the root element of a document.
    ///
    /// Note: this can be false even if `.parent_element()` is `None`
    /// if the parent node is a `DocumentFragment`.
    fn is_root(&self) -> bool { false }
}


impl Selector {
    /// Returns whether the given element matches this selector.
    // #[inline]
    pub fn matches(&self, element: &EntityElement) -> bool {
        let mut context = matching::MatchingContext::new(
            matching::MatchingMode::Normal,
            None,
            None,
            QuirksMode::NoQuirks,
        );
        matching::matches_selector(&self.0, 0, None, element, &mut context, &mut |_, _| {})
    }

    /// Return the specificity of this selector.
    pub fn specificity(&self) -> Specificity {
        Specificity(self.0.specificity())
    }
}

impl ::std::str::FromStr for Selectors {
    type Err = ();
    #[inline]
    fn from_str(s: &str) -> Result<Selectors, ()> {
        Selectors::compile(s)
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.to_css(f)
    }
}

impl std::fmt::Display for Selectors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut iter = self.0.iter();
        let first = iter.next()
            .expect("Empty Selectors, should contain at least one selector");
        let _ = (first.0.to_css(f))?;
        for selector in iter {
            let _ = (f.write_str(", "))?;
            let _ = (selector.0.to_css(f))?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Debug for Selectors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

pub struct StyleSystem;
impl<'a> System<'a> for StyleSystem {
    type SystemData = (
        ReadStorage<'a, Pseudo>,
        ReadStorage<'a, StyleBackground>,
        WriteStorage<'a, crate::rendering::Material>,
    );

    #[allow(dead_code)]
    fn run(&mut self, (pseudo, bg, mut mat): Self::SystemData) {
        //  let selectors = Selectors::compile(":hover").unwrap();
        //  let mut top = TopLevelRuleParser {};
        //  let stylesheet = cssparser::RuleListParser::new_for_stylesheet(&mut top, parser: P).expect("Wasn't a valid stylesheet");

        //  selectors.matches(element: &EntityElement)
        //  println!("{:?}", selectors);
        for (pseudo, bg, mut mat) in (pseudo.maybe(), &bg, &mut mat).join() {
            mat.color = if pseudo.map_or(false, |v| v.hover) {
                bg.color
            } else {
                bg.color / 2
            };
        }
    }
}

impl StyleSystem {
    pub fn new() -> Self {
        Self {}
    }
}


#[derive(Debug)]
pub struct StyleBackground {
    pub color: cgmath::Vector4<u8>,
}

impl specs::Component for StyleBackground {
    type Storage = DenseVecStorage<Self>;
}

impl StyleBackground {
    pub fn from_color(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            color: cgmath::Vector4::new(r, g, b, a),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Pseudo {
    pub hover: bool,
}

impl specs::Component for Pseudo {
    type Storage = DenseVecStorage<Self>;
}


#[cfg(test)]
mod tests {
    use crate::style_system::*;
    #[test]
    fn match_hover() {
        let s = Selectors::compile(":hover").unwrap();

        assert_eq!(false, s.matches(&EntityElement::new("a".into()).with_hover(false) ));
        assert_eq!(true, s.matches(&EntityElement::new("a".into()).with_hover(true)));
    }

    #[test]
    fn match_type() {
        let s = Selectors::compile("A").unwrap();

        assert_eq!(false, s.matches(&EntityElement::new("B".into())));
        assert_eq!(true, s.matches(&EntityElement::new("A".into())));
    }

    #[test]
    fn match_parent() {
        let s = Selectors::compile("A B").unwrap();

        assert_eq!(false, s.matches(&EntityElement::new("B".into())));
        assert_eq!(false, s.matches(&EntityElement::new("A".into())));
    }

    #[test]
    fn match_id() {
        let s = Selectors::compile("#id").unwrap();

        assert_eq!(false, s.matches(&EntityElement::new("B".into()).with_id("asd".into())));
        assert_eq!(true, s.matches(&EntityElement::new("A".into()).with_id("id".into())));
    }

    #[test]
    fn match_class() {
        let s = Selectors::compile(".a").unwrap();

        assert_eq!(false, s.matches(&EntityElement::new("X".into()).add_class("b".into())));
        assert_eq!(true, s.matches(&EntityElement::new("X".into()).add_class("a".into())));
    }
}
