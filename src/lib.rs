#![feature(universal_impl_trait)]
#![feature(conservative_impl_trait)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

use std::{fmt, io};
use std::fmt::Arguments;

pub mod html;

pub trait Renderer {
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.write_raw(data)
    }
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        self.write(format!("{}", fmt).as_bytes())
    }
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.write(s.as_bytes())
    }
    fn write_raw(&mut self, data: &[u8]) -> io::Result<()>;

    fn write_raw_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        self.write_raw(format!("{}", fmt).as_bytes())
    }
    fn write_raw_str(&mut self, s: &str) -> io::Result<()> {
        self.write_raw(s.as_bytes())
    }
}

pub trait Render {
    fn render(self, &mut Renderer) -> io::Result<()>;
}

impl<T: Render> Render for Vec<T> {
    fn render(mut self, r: &mut Renderer) -> io::Result<()> {
        for t in self.drain(..) {
            t.render(r)?;
        }
        Ok(())
    }
}

impl Render for () {
    fn render(self, _: &mut Renderer) -> io::Result<()> {
        Ok(())
    }
}

impl Render for String {
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        r.write_raw(self.as_bytes())
    }
}

impl Render for usize {
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        r.write_raw_fmt(format_args!("{}", self))
    }
}

impl<'a> Render for &'a str {
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        r.write_str(self)
    }
}

impl<'a> Render for fmt::Arguments<'a> {
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        r.write_fmt(self)
    }
}

impl<'a> Render for &'a fmt::Arguments<'a> {
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        r.write_fmt(*self)
    }
}

impl<A> Render for (A,)
where
    A: Render,
{
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)
    }
}

impl<A, B> Render for (A, B)
where
    A: Render,
    B: Render,
{
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)
    }
}

impl<A, B, C> Render for (A, B, C)
where
    A: Render,
    B: Render,
    C: Render,
{
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)
    }
}


impl<F> Render for F
where
    F: FnOnce(&mut Renderer) -> io::Result<()>,
{
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        self(r)
    }
}
