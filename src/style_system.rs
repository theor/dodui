#![allow(dead_code)]

use specs::prelude::*;
use selectors::*;
use selectors::parser::*;

use selectors::parser::{SelectorImpl, Parser, SelectorList, Selector as GenericSelector, NonTSPseudoClass};
use selectors::context::*;
use selectors::matching::ElementSelectorFlags;
use selectors::attr::{AttrSelectorOperation, NamespaceConstraint, CaseSensitivity};
use cssparser::{self, ToCss, CowRcStr, SourceLocation, ParseError};

use std::sync::Mutex;

use crate::transform::Parent;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Sym(string_interner::Sym);

impl Sym {
    pub fn resolve<'a>(&'a self) -> Option<String> {
        let si: &string_interner::DefaultStringInterner = &STRING_INTERNER.lock().unwrap();
        si.resolve(self.0).map(|x| x.to_owned())
    }
}

lazy_static! {
    static ref STRING_INTERNER: Mutex<string_interner::DefaultStringInterner> = {
        let m = string_interner::DefaultStringInterner::new();
        Mutex::new(m)
    };
}

impl std::fmt::Display for Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match STRING_INTERNER.lock().unwrap().resolve(self.0) {
            Some(x) => { x.fmt(f) },
            None => panic!("resolve"),
        }
    }
}

// impl std::borrow::Borrow<Sym> for String {
//     fn borrow(&self) -> &Sym {
        
//         Sym(STRING_INTERNER.lock().unwrap().get_or_intern(self.as_str()))
//     }
// }

impl<'a> std::convert::From<&'a str> for Sym {
    fn from(s:&'a str) -> Self {
        let sym = STRING_INTERNER.lock().unwrap().get_or_intern(s);
        Self(sym)
    }
}

impl<'a> std::convert::From<String> for Sym {
    fn from(s:String) -> Self {
        let sym = STRING_INTERNER.lock().unwrap().get_or_intern(s);
        Self(sym)
    }
}

#[derive(Debug, Clone)]
pub struct KuchikiSelectors;

impl SelectorImpl for KuchikiSelectors {
    type AttrValue = String;
    type Identifier = Sym;//LocalName;
    type ClassName = Sym;//LocalName;
    type LocalName = Sym;//LocalName;
    type NamespacePrefix = String;//LocalName;
    type NamespaceUrl = String;//Namespace;
    type BorrowedNamespaceUrl = str;//Namespace;
    type BorrowedLocalName = Sym;//LocalName;

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
pub struct EElement {
    id: Option<Sym>,
    typeid: Sym,
    classes: std::collections::HashSet<Sym>,
    pseudo: Pseudo,
}

impl specs::Component for EElement {
    type Storage = DenseVecStorage<Self>;
}

impl EElement {
    pub fn new(typeid: String) -> Self {
        EElement {
            typeid: typeid.into(),
            id: None,
            pseudo: Pseudo { hover: false},
            classes: Default::default(),
        }
    }

    pub fn with_hover(self, hover: bool) -> Self {
        EElement {
            pseudo: Pseudo { hover },
            ..self
        }
    }

    pub fn with_id(self, id: String) -> Self {
        EElement {
            id: Some(id.into()),
            ..self
        }
    }

    pub fn add_class(mut self, cl: String) -> Self {
        use std::iter::FromIterator;
        let mut classes = std::collections::HashSet::from_iter(self.classes.drain());
        classes.insert(cl.into());
        EElement {
            classes,
            ..self
        }
    }
}

type EntityElementStorage<'a> = (&'a ReadStorage<'a, EElement>, &'a ReadStorage<'a, Parent>);

#[derive(Clone)]
pub struct EntityElement<'a>(EntityElementStorage<'a>, Entity);

impl<'a> std::fmt::Debug for EntityElement<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", &self.1)
    }
}

impl<'a> EntityElement<'a> {
    // pub fn new(e: &'a EElement) -> Self {
    //     Self(e, None)
    // }

    pub fn eelt(&self) -> &'a EElement { (self.0).0.get(self.1).unwrap() }
}

impl<'a> Element for EntityElement<'a> {
    type Impl = KuchikiSelectors;

    /// Converts self into an opaque representation.
    fn opaque(&self) -> OpaqueElement { OpaqueElement::new(&self) }

    // TODO
    fn parent_element(&self) -> Option<Self> { (self.0).1.get(self.1).map(|x| EntityElement(self.0, x.entity)) }

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
        &self.eelt().typeid
    }

    /// Empty string for no namespace
    fn namespace(&self) -> &<Self::Impl as SelectorImpl>::BorrowedNamespaceUrl { &"" }

    fn attr_matches(
        &self,
        _ns: &NamespaceConstraint<&<Self::Impl as SelectorImpl>::NamespaceUrl>,
        _local_name: &<Self::Impl as SelectorImpl>::LocalName,
        _operation: &AttrSelectorOperation<&<Self::Impl as SelectorImpl>::AttrValue>,
    ) -> bool {
        unimplemented!();
    }

    fn match_non_ts_pseudo_class<F>(
        &self,
        pc: &<Self::Impl as SelectorImpl>::NonTSPseudoClass,
        _context: &mut MatchingContext<Self::Impl>,
        _flags_setter: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags) {
            match pc {
                PseudoClass::Hover => self.eelt().pseudo.hover,
                _ => false,
            }
        }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as SelectorImpl>::PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
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
        _case_sensitivity: CaseSensitivity,
    ) -> bool { self.eelt().id.as_ref().map_or(false, |x| x == id) }

    fn has_class(
        &self,
        name: &<Self::Impl as SelectorImpl>::ClassName,
        _case_sensitivity: CaseSensitivity,
    ) -> bool { self.eelt().classes.contains(name) }

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
        Entities<'a>,
        ReadExpect<'a, crate::manager::ResourceManager>,
        ReadStorage<'a, Pseudo>,
        ReadStorage<'a, crate::transform::Parent>,
        ReadStorage<'a, EElement>,
        WriteStorage<'a, StyleBackground>,
        WriteStorage<'a, crate::rendering::Material>,
    );

    #[allow(dead_code)]
    fn run(&mut self, (entities, res, pseudo, parent, ee, mut bg, mut mat): Self::SystemData) {
        use crate::manager::*;

        let key = SimpleKey::Path(("style/style.css").into());
        let stylesheet = match res.get::<crate::styling::Stylesheet>(&key) {
            Ok(css) => css,
            e => { eprintln!("{:?}", e); return},
        };

        let mut matching_entities: Vec<Entity> = Vec::new();
        matching_entities.extend((&entities, &ee).join().map(|x| x.0));
        for e in matching_entities.iter() {
            for rule in stylesheet.borrow().0.iter() {
                if rule.selectors.matches(&EntityElement((&ee, &parent), *e)) {
                    for declaration in rule.declarations.iter(){
                        match declaration.property.as_ref() {
                            "background" => { bg.get_mut(*e).unwrap().color = declaration.value.color().unwrap().into() },
                            _ => unimplemented!(),
                        }
                    }
                }
            }
        }

        for (bg, mut mat) in (&bg, &mut mat).join() {
            mat.color = bg.color;
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
    use crate::transform::Parent;

    fn world() -> World {
        let mut w = specs::World::new();
        w.register::<Parent>();
        w.register::<EElement>();
        w
    }

    fn check(res: bool, w: &mut World, s: &Selectors, e: EElement, parent: Option<Entity>) -> Entity {
        let mut e = w.create_entity().with(e);
        if let Some(parent) = parent {
            e = e.with(Parent { entity: parent });
        }
        let e = e.build();
        let (ee,p) : (ReadStorage<EElement>, ReadStorage<Parent>) = w.system_data();

        assert_eq!(res, s.matches(&EntityElement((&ee,&p), e)));

        e
    }

    #[test]
    fn match_hover() {
        let s = Selectors::compile(":hover").unwrap();
        let mut w = world();

        check(false, &mut w, &s, EElement::new("B".into()).with_hover(false), None);
        check(true, &mut w, &s, EElement::new("B".into()).with_hover(true), None);
    }

    #[test]
    fn match_type() {
        let s = Selectors::compile("A").unwrap();
        let mut w = world();

        check(false, &mut w, &s, EElement::new("B".into()), None);
        check(true, &mut w, &s, EElement::new("A".into()), None);
    }

    #[test]
    fn match_parent() {
        let s = Selectors::compile("A B").unwrap();

        let mut w = world();

        let ea = check(false, &mut w, &s, EElement::new("A".into()), None);
        let eb = check(true, &mut w, &s, EElement::new("B".into()), Some(ea));
        let eb2 = check(true, &mut w, &s, EElement::new("B".into()), Some(eb));
        let _ec = check(false, &mut w, &s, EElement::new("C".into()), Some(eb2));
    }

    #[test]
    fn match_id() {
        let s = Selectors::compile("#id").unwrap();
        let mut w = world();

        check(false, &mut w, &s, EElement::new("B".into()).with_id("asd".into()), None);
        check(true, &mut w, &s, EElement::new("A".into()).with_id("id".into()), None);
    }

    #[test]
    fn match_class() {
        let s = Selectors::compile(".a").unwrap();
        let mut w = world();

        check(false, &mut w, &s, EElement::new("X".into()).add_class("b".into()), None);
        check(true, &mut w, &s, EElement::new("X".into()).add_class("a".into()), None);
    }
}
