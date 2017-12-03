use std::io;
use std::fmt;
use std::borrow::Cow;

use Render;

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

    fn write_raw_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.io.write_fmt(fmt)
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
    fn render(self, r: &mut super::Renderer) -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(&*self.tag)?;
        for (k, v) in self.attrs {
            r.write_raw_str(" ")?;
            r.write_str(&*k)?;
            r.write_raw_str("=\"")?;
            r.write_str(&*v)?;
            r.write_raw_str("\"")?;
        }
        r.write_raw_str(">")?;
        r.write_raw_str("</")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")
    }
}

impl Render for BareTag {
    fn render(self, r: &mut super::Renderer) -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")?;
        r.write_raw_str("</")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")
    }
}

impl<I: Render> Render for FinalTag<I> {
    fn render(self, r: &mut super::Renderer) -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(&*self.tag)?;
        for (k, v) in self.attrs {
            r.write_raw_str(" ")?;
            r.write_str(&*k)?;
            r.write_raw_str("=\"")?;
            r.write_str(&*v)?;
            r.write_raw_str("\"")?;
        }

        r.write_raw_str(">")?;
        self.inn.render(r)?;
        r.write_raw_str("</")?;
        r.write_raw_str(&*self.tag)?;
        r.write_raw_str(">")
    }
}

impl Tag {
    pub fn attr(self, key: &'static str, val: &'static str) -> Tag {
        let Tag { tag, mut attrs } = self;
        attrs.push((CowStr::from(key), CowStr::from(val)));
        Tag {
            tag: tag,
            attrs: attrs,
        }
    }
    pub fn class(self, val: &'static str) -> Tag {
        self.attr("class", val)
    }
    pub fn id(self, val: &'static str) -> Tag {
        self.attr("id", val)
    }
}

impl BareTag {
    pub fn attr(&self, key: &'static str, val: &'static str) -> Tag {
        Tag {
            tag: self.tag.into(),
            attrs: vec![(key.into(), CowStr::from(val))],
        }
    }
    pub fn class(self, val: &'static str) -> Tag {
        self.attr("class", val)
    }
    pub fn id(self, val: &'static str) -> Tag {
        self.attr("id", val)
    }
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

impl_tag!(html);
impl_tag!(head);
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
