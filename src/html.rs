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
    inn: Box<Render>,
}

pub struct FinalTag {
    tag: CowStr,
    attrs: Vec<(CowStr, CowStr)>,
    inn: Box<Render>,
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
        self.inn.render(r)?;
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

impl Tag {
    fn bar(&self) {
        println!("calling bar");
    }

    fn attr(self, key: &'static str, val: &'static str) -> Tag {
        let Tag {
            tag,
            mut attrs,
            inn,
        } = self;
        attrs.push((CowStr::from(key), CowStr::from(val)));
        Tag {
            tag: tag,
            attrs: attrs,
            inn: inn,
        }
    }
}

impl BareTag {
    fn attr(&self, key: &'static str, val: &'static str) -> Tag {
        Tag {
            tag: self.tag.into(),
            attrs: vec![(key.into(), CowStr::from(val))],
            inn: Box::new(()),
        }
    }
}

impl<A: Render + 'static> FnOnce<(A,)> for Tag {
    type Output = Tag;
    extern "rust-call" fn call_once(mut self, args: (A,)) -> Self::Output {
        self.inn = Box::new(args.0);
        self
    }
}

impl<A: Render + 'static> FnOnce<(A,)> for BareTag {
    type Output = FinalTag;
    extern "rust-call" fn call_once(self, args: (A,)) -> Self::Output {
        FinalTag {
            tag: self.tag.into(),
            attrs: vec![],
            inn: Box::new(args.0),
        }
    }
}

const div: BareTag = BareTag { tag: "div" };
const p: BareTag = BareTag { tag: "p" };

fn template() -> impl Render {
    div(p);
}

fn main() {
    //div.attr("foo", "bar");
    //div.attr("foo", "bar")(());

    let _ = template();
    //div("foo, bar");
}

fn wrap_in_tag(tag: &'static str, inner: impl Render) -> impl Render {
    move |r: &mut super::Renderer| -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(tag)?;
        r.write_raw_str(">")?;
        inner.render(r)?;
        r.write_raw_str("</")?;
        r.write_raw_str(tag)?;
        r.write_raw_str(">")
    }
}
