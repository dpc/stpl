use std::io;
use std::fmt;
use std::borrow::Cow;

use serde;
use Render;
use super::Fn;
use std;

/// A HTML Template
///
/// It consists of unique key used to identify the template
/// across dynamic rendering calls, and a function accepting
/// one (de-)serializable data, and returning a value of `Render` type.
///
/// Typically you will want a list of global functions
/// for all templates (eg. all html pages of your web-app)
/// defined. Eg.
///
/// ```
/// pub fn home_tpl() -> impl Template {
///    Template::new("home", tpl::home::page)
/// }
/// ```
pub struct Template<F, A> {
    key: &'static str,
    f: F,
    _a: std::marker::PhantomData<A>,
}

impl<F, A, R> Template<F, A>
where
    F: std::ops::Fn(&A) -> R,
    R: Render,
    A: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    pub fn new(key: &'static str, f: F) -> Self {
        Template {
            key: key,
            f: f,
            _a: std::marker::PhantomData,
        }
    }
}

impl<F, A, R> ::Template for Template<F, A>
where
    F: std::ops::Fn(&A) -> R,
    R: Render,
    A: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Argument = A;
    fn key(&self) -> &'static str {
        self.key
    }
    fn render(&self, argument: &Self::Argument, io: &mut io::Write) -> io::Result<()> {
        let render = self.f.call((argument,));
        render.render(&mut Renderer::new(io))?;
        Ok(())
    }
}

pub trait RenderExt: Render {
    fn render_to_vec(&self) -> Vec<u8> {
        let mut v: Vec<u8> = vec![];
        self.render(&mut Renderer::new(&mut v)).unwrap();
        v
    }

    fn renderto_string(&self) -> String {
        String::from_utf8_lossy(&self.render_to_vec()).into()
    }
}

impl<T: Render + ?Sized> RenderExt for T {}

pub struct Renderer<T> {
    io: T,
    tmp: Vec<u8>,
}

impl<T: io::Write> Renderer<T> {
    pub fn new(t: T) -> Self {
        Renderer { io: t, tmp: vec![] }
    }
}

impl<T: io::Write> super::Renderer for Renderer<T> {
    fn write_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.io.write_all(data)
    }

    fn write_raw_fmt(&mut self, fmt: &fmt::Arguments) -> io::Result<()> {
        self.io.write_fmt(*fmt)
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.tmp.clear();

        for c in data {
            match *c as char {
                '&' => self.tmp.extend_from_slice("&amp;".as_bytes()),
                '<' => self.tmp.extend_from_slice("&lt;".as_bytes()),
                '>' => self.tmp.extend_from_slice("&gt;".as_bytes()),
                '"' => self.tmp.extend_from_slice("&quot;".as_bytes()),
                '\'' => self.tmp.extend_from_slice("&#x27;".as_bytes()),
                '/' => self.tmp.extend_from_slice("&#x2F;".as_bytes()),
                // Additional one for old IE (unpatched IE8 and below)
                // See https://github.com/OWASP/owasp-java-encoder/wiki/Grave-Accent-Issue
                '`' => self.tmp.extend_from_slice("&#96;".as_bytes()),
                _ => self.tmp.push(*c),
            }
        }

        self.io.write_all(&self.tmp)?;

        Ok(())
    }
}

type CowStr = Cow<'static, str>;

pub struct BareTag {
    tag: &'static str,
}

pub struct Tag {
    tag: CowStr,
    attrs: Vec<(CowStr, CowStr)>,
}

pub struct FinalTag<I> {
    tag: CowStr,
    attrs: Vec<(CowStr, CowStr)>,
    inn: I,
}

impl Render for Tag {
    fn render(&self, r: &mut super::Renderer) -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(&*self.tag)?;
        for &(ref k, ref v) in self.attrs.iter() {
            r.write_raw_str(" ")?;
            r.write_raw_str(&*k)?;
            r.write_raw_str("=\"")?;
            r.write_raw_str(&*v)?;
            r.write_raw_str("\"")?;
        }
        r.write_raw_str(">")?;
        r.write_raw_str("</")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")
    }
}

impl Render for BareTag {
    fn render(&self, r: &mut super::Renderer) -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")?;
        r.write_raw_str("</")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")
    }
}

impl<I: Render> Render for FinalTag<I> {
    fn render(&self, r: &mut super::Renderer) -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(&*self.tag)?;
        for &(ref k, ref v) in self.attrs.iter() {
            r.write_raw_str(" ")?;
            r.write_raw_str(&*k)?;
            r.write_raw_str("=\"")?;
            r.write_raw_str(&*v)?;
            r.write_raw_str("\"")?;
        }

        r.write_raw_str(">")?;
        self.inn.render(r)?;
        r.write_raw_str("</")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")
    }
}

macro_rules! impl_attr {
    ($t:ident) => {
        pub fn $t<V : Into<CowStr>>(self, val: V) -> Tag {
            self.attr(stringify!($t), val)
        }
    }
}

macro_rules! impl_attr_all {
    () => (
        impl_attr!(class);
        impl_attr!(id);
        impl_attr!(charset);
        impl_attr!(content);
        impl_attr!(name);
        impl_attr!(href);
        impl_attr!(rel);
        impl_attr!(src);
        impl_attr!(integrity);
        impl_attr!(crossorigin);
        impl_attr!(role);
        impl_attr!(method);
        impl_attr!(action);
        impl_attr!(placeholder);
        impl_attr!(value);
        impl_attr!(alt);
        impl_attr!(style);

        pub fn type_<V : Into<CowStr>>(self, val: V) -> Tag {
            self.attr("type", val)
        }
        pub fn data_toggle<V : Into<CowStr>>(self, val: V) -> Tag {
            self.attr("data-toggle", val)
        }
        pub fn data_target(self, val: &'static str) -> Tag {
            self.attr("data-target", val)
        }
        pub fn aria_controls(self, val: &'static str) -> Tag {
            self.attr("aria-controls", val)
        }
        pub fn aria_expanded(self, val: &'static str) -> Tag {
            self.attr("aria-expanded", val)
        }
        pub fn aria_label(self, val: &'static str) -> Tag {
            self.attr("aria-label", val)
        }
        pub fn aria_haspopup(self, val: &'static str) -> Tag {
            self.attr("aria-haspopup", val)
        }
        pub fn aria_labelledby(self, val: &'static str) -> Tag {
            self.attr("aria-labelledby", val)
        }
        pub fn aria_current(self, val: &'static str) -> Tag {
            self.attr("aria-current", val)
        }
    )
}

impl Tag {
    pub fn attr<K: Into<CowStr>, V: Into<CowStr>>(self, key: K, val: V) -> Tag {
        let Tag { tag, mut attrs } = self;
        attrs.push((key.into(), val.into()));
        Tag {
            tag: tag,
            attrs: attrs,
        }
    }

    impl_attr_all!();
}

impl BareTag {
    pub fn attr<K: Into<CowStr>, V: Into<CowStr>>(self, key: K, val: V) -> Tag {
        Tag {
            tag: self.tag.into(),
            attrs: vec![(key.into(), val.into())],
        }
    }
    impl_attr_all!();
}

impl<A: Render + 'static> FnOnce<(A,)> for Tag {
    type Output = FinalTag<A>;
    extern "rust-call" fn call_once(self, args: (A,)) -> Self::Output {
        FinalTag {
            tag: self.tag,
            attrs: self.attrs,
            inn: args.0,
        }
    }
}

impl<A: Render + 'static> FnOnce<(A,)> for BareTag {
    type Output = FinalTag<A>;
    extern "rust-call" fn call_once(self, args: (A,)) -> Self::Output {
        FinalTag {
            tag: self.tag.into(),
            attrs: vec![],
            inn: args.0,
        }
    }
}

/// Implement a Render function wrapping in a simple tag
macro_rules! impl_tag {
    ($t:ident) => {
        #[allow(non_upper_case_globals)]
        pub const $t: BareTag = BareTag { tag: stringify!($t) };
    }
}

pub fn doctype(t: &'static str) -> impl Render {
    Fn(move |r: &mut super::Renderer| {
        r.write_raw(b"<!DOCTYPE ")?;
        r.write_raw_str(t)?;
        r.write_raw(b">")
    })
}

#[derive(Copy, Clone)]
pub struct Nbsp;

#[allow(non_upper_case_globals)]
pub const nbsp: Nbsp = Nbsp;

impl Render for Nbsp {
    fn render(&self, r: &mut super::Renderer) -> io::Result<()> {
        r.write_raw_str("&nbsp;")
    }
}

pub fn raw<T: Render>(x: T) -> impl Render {
    Fn(move |r: &mut super::Renderer| x.render(&mut super::RawRenderer(r)))
}

impl_tag!(html);
impl_tag!(head);
impl_tag!(meta);
impl_tag!(title);
impl_tag!(body);
impl_tag!(div);
impl_tag!(section);
impl_tag!(h1);
impl_tag!(h2);
impl_tag!(h3);
impl_tag!(h4);
impl_tag!(h5);
impl_tag!(li);
impl_tag!(ul);
impl_tag!(ol);
impl_tag!(p);
impl_tag!(span);
impl_tag!(b);
impl_tag!(i);
impl_tag!(u);
impl_tag!(string);
impl_tag!(pre);
impl_tag!(link);
impl_tag!(script);
impl_tag!(main);
impl_tag!(nav);
impl_tag!(a);
impl_tag!(form);
impl_tag!(button);
impl_tag!(input);
impl_tag!(img);
impl_tag!(blockquote);
impl_tag!(footer);

// vim: foldmethod=marker foldmarker={{{,}}}
