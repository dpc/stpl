//! stpl - Super template library for Rust
//!
//! This version of `stpl` is a Proof of Concept. If you like it, or dislike it
//! please be vocal about it.
//!
//! `stpl` goals:
//!
//! * no ugly macros; actually, no macros at all;
//! * no text-files with weird syntax; Rust only!
//! * still, nice syntax and support for run-time templates (magic!)
//!
//! Nits:
//!
//! * uses 4 `nightly` unstable features
//!
//! # Help
//!
//! Please see `./playground` subdirectory for example usage.

#![feature(universal_impl_trait)]
#![feature(conservative_impl_trait)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate bincode;
extern crate serde;

use std::{fmt, io};
use std::fmt::Arguments;
use std::io::{Read, Write};


/// HTML rendering functions
pub mod html;

/// Rendering destination
///
/// Takes care of escaping and such.
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

/// A type that can be rendered to `Renderer`
///
/// This can be generally thought as a `template`, with it's data,
/// that is ready to render itself into the `Renderer`.
///
/// It is defined for bunch of `std` types. Please send PR if
/// something is missing.
///
/// You can impl it for your own types too. You usually compose it
/// from many other `impl Render` data.
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

impl<T: Render> Render for Box<T> {
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        (*self).render(r)?;
        Ok(())
    }
}



impl Render for () {
    fn render(self, _: &mut Renderer) -> io::Result<()> {
        Ok(())
    }
}

impl<R: Render> Render for Option<R> {
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        if let Some(s) = self {
            s.render(r)?
        }
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

/// Use to wrap closures with
pub struct Fn<F>(pub F);

impl<F> Render for Fn<F>
where
    F: FnOnce(&mut Renderer) -> io::Result<()>,
{
    fn render(self, r: &mut Renderer) -> io::Result<()> {
        self.0(r)
    }
}


fn handle_dynamic_impl<F, A, R>(f: F) -> io::Result<()>
where
    for<'a> F: FnOnce<(&'a A,), Output = R>,
    R: Render,
    for<'de> A: serde::Deserialize<'de>,
{
    let mut v = vec![];
    std::io::stdin().read_to_end(&mut v)?;
    let arg: A = bincode::deserialize(&v[..])
        .map_err(|_e| io::Error::new(io::ErrorKind::Other, "Deserialization error"))?;
    let tpl = f.call_once((&arg,));
    v.clear();

    tpl.render(&mut html::Renderer::new(&mut v))?;
    std::io::stdout().write_all(&v)?;
    Ok(())
}

/// `handle_dynamic` handle
pub struct HandleDynamic;

pub fn handle_dynamic() -> HandleDynamic {
    HandleDynamic
}

impl HandleDynamic {
    pub fn html<F, A, R>(&self, key: &str, f: F)
    where
        for<'a> F: FnOnce<(&'a A,), Output = R>,
        R: Render,
        for<'de> A: serde::Deserialize<'de>,
    {
        // TODO: optimize, don't fetch every time?
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
}

impl std::ops::Drop for HandleDynamic {
    fn drop(&mut self) {
        if let Ok(var_key) = std::env::var("RUST_STPL_DYNAMIC_TEMPLATE_KEY") {
            if !var_key.is_empty() {
                eprintln!("Couldn't find dynamic template by key: {}", var_key);
                std::process::exit(68);
            }
        }
    }
}



/// Call a template dynamically (with ability to update at runtime)
///
/// Make sure to put `handle_dynamic` at the very beginning of your
/// binary if you want to use it.
///
/// `data` type must be the same as template expects.
///
/// The template will evaluate in another process, so you can't rely
/// on value of globals, and such, but otherwise it's transparent.
///
/// It works by serializing `data` and passing it to a child process.
/// The child process is the current binary, with environment variable
/// pointing to the right template. `handle_dynamic` will detect
/// being a dynamic-template-child, deserialize `data`, render
/// the template and write the output to `stdout`. This will
/// be used as a transparent Template.
///
/// TODO: This returning `impl Render` doesn't make sense.
pub fn call_dynamic<A>(key: &str, data: &A) -> impl Render + 'static
where
    A: serde::Serialize,
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

    // TODO: Sending could be done in a separate thread too.
    {
        let stdin = child.stdin.as_mut().expect("failed to get stdin");
        stdin.write_all(&encoded).expect("failed to write to stdin");
        stdin.flush().expect("failed to flush stdin");
    }
    child.stdin = None;

    Fn(move |r: &mut Renderer| {
        let out = child.wait_with_output()?;
        if out.status.success() {
            r.write_raw(&out.stdout[..])
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Dynamic template process failed",
            ))
        }
    })
}
