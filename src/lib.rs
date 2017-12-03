#![feature(universal_impl_trait)]
#![feature(conservative_impl_trait)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate bincode;
extern crate serde;

use std::{fmt, io};
use std::fmt::Arguments;
use std::io::{Read, Write};


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
        self.0.render(r)?;
        self.1.render(r)
    }
}

impl<A, B, C> Render for (A, B, C)
where
    A: Render,
    B: Render,
    C: Render,
{
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)?;
        self.1.render(r)?;
        self.2.render(r)
    }
}

impl<A, B, C, D> Render for (A, B, C, D)
where
    A: Render,
    B: Render,
    C: Render,
    D: Render,
{
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)?;
        self.1.render(r)?;
        self.2.render(r)?;
        self.3.render(r)
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


fn handle_dynamic_impl<F, A>(f: F) -> io::Result<()>
where
    F: FnOnce<(A,)>,
    A: for<'de> serde::Deserialize<'de>,
    <F as std::ops::FnOnce<(A,)>>::Output: Render,
{
    let mut v = vec![];
    std::io::stdin().read_to_end(&mut v)?;
    let arg: A = bincode::deserialize(&v[..])
        .map_err(|_e| io::Error::new(io::ErrorKind::Other, "Deserialization error"))?;
    let tpl = f.call_once((arg,));
    v.clear();

    tpl.render(&mut html::Renderer::new(&mut v))?;
    std::io::stdout().write_all(&v)?;
    Ok(())
}

pub fn handle_dynamic<F, A>(key: &str, f: F)
where
    F: FnOnce<(A,)>,
    A: for <'de> serde::Deserialize<'de>,
    <F as std::ops::FnOnce<(A,)>>::Output: Render,
{
    if let Ok(var_key) = std::env::var("RUST_STPL_DYNAMIC_TEMPLATE_KEY") {
        if var_key.as_str() == key {
            match handle_dynamic_impl(f) {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    eprintln!("Dynamic template process failed: {:?}", e);
                    std::process::exit(67);
                }
            }
        }
    }
}



pub fn call_dynamic<A>(key: &str, data: A) -> impl Render + 'static
where
    A:  serde::Serialize,
{
    let encoded: Vec<u8> = bincode::serialize(&data, bincode::Infinite).unwrap();

    use std::process::{Command, Stdio};
    use std::io::Write;

    let mut child = Command::new(std::env::args_os().next().unwrap())
        .env("RUST_STPL_DYNAMIC_TEMPLATE_KEY", key)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    {
        // limited borrow of stdin
        let stdin = child.stdin.as_mut().expect("failed to get stdin");
        stdin.write_all(&encoded).expect("failed to write to stdin");
        stdin.flush().expect("failed to flush stdin");
    }
    child.stdin = None;

    move |r : &mut Renderer| {
        let out = child
            .wait_with_output()?;
        if out.status.success() {
            r.write_raw(&out.stdout[..])
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Dynamic template process failed"))
        }

    }

}
