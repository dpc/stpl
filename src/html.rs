use std::io;
use std::fmt;

use {Render};

pub struct Renderer<T>{
    io: T,
    tmp: Vec<u8>,
    }

impl<T: io::Write> Renderer<T> {
    pub fn new(t: T) -> Self {
        Renderer {
            io: t,
            tmp: vec![],
        }
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


/// Implement a Render function wrapping in a simple tag
macro_rules! impl_tag {
    ($t:ident) => {
        pub fn $t(inner: impl Render) -> impl Render {
            wrap_in_tag(stringify!($t), inner)
        }
    };
}

impl_tag!(html);
impl_tag!(body);
impl_tag!(h1);
impl_tag!(li);
impl_tag!(ul);
impl_tag!(ol);
impl_tag!(p);

fn wrap_in_tag(tag : &'static str, inner: impl Render) -> impl Render {
    move |r : &mut super::Renderer| -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(tag)?;
        r.write_raw_str(">")?;
        inner.render(r)?;
        r.write_raw_str("</")?;
        r.write_raw_str(tag)?;
        r.write_raw_str(">")
    }
}

