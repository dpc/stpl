//! ## stpl - Super template library for Rust
//!
//! `stpl` is a plain-Rust-only template library with some neat properties and
//! features.
//!
//! ## Main idea
//!
//! In `stpl` there are no magic macros or DSLs, and no clunky
//! text-files with weird syntax. Everything is just normal, easy
//! to understand Rust code.
//!
//! Let's take a look at a real-life example from the pilot project: an HTML
//! based-skeleton template for a Bootstrap-based UI.
//!
//! ```ignore
//! pub fn base<C: Render + 'static>(data: &Data, content: C) -> impl Render {
//!     (
//!         doctype("html"),
//!         html((
//!             head((
//!                 meta.charset("utf-8"),
//!                 meta.name("viewport").content("width=device-width, initial-scale=1, shrink-to-fit=no"),
//!                 meta.name("description").content(""),
//!                 meta.name("author").content("Dawid Ciężarkiewicz"),
//!                 title(data.title.clone()),
//!                 (
//!                     link.rel("icon").href("/static/favicon.ico"),
//!                     link.rel("stylesheet").href("/static/theme/flatly/bootstrap.min.css"),
//!                     link.rel("stylesheet").href("/static/theme/starter-template.css"),
//!                 )
//!             )),
//!             body((
//!                 navbar(data),
//!                 main
//!                     .id("main")
//!                     .role("main")
//!                     .class("container mb-5")(
//!                     content,
//!                 ),
//!                 (
//!                 script.src("https://code.jquery.com/jquery-3.2.1.min.js").crossorigin("anonymous"),
//!                 script.src("https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.3/umd/popper.min.js")
//!                     .integrity("sha384-vFJXuSJphROIrBnz7yo7oB41mKfc8JzQZiCq4NCceLEaO4IHwicKwpJf9c9IpFgh")
//!                     .crossorigin("anonymous"),
//!                 script.src("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/js/bootstrap.min.js")
//!                     .integrity("sha384-alpBpkh1PFOepccYVYDB4do5UnbKysX5WZXm3XxPqe5iKTfUKjNkCk9SaVuEZflJ")
//!                     .crossorigin("anonymous"),
//!                 script.type_("text/javascript")(
//!                     raw(include_str!("white-icon.js"))
//!                 ),
//!                 )
//!             ))
//!         ))
//!     )
//! }
//! ```
//!
//! It is just a function. There is no magic, no macros, no text files involved.
//! The whole template was formatted with `rustfmt` just like a normal Rust code.
//!
//! The function accepts arguments:
//!
//! * `data: Data` containing information how to "fill the blanks", and
//! * `content: Render` - sub-template value that will be used as main page content.
//!
//! The function returns `Render` value that can be rendered as a string or bytes, or
//! composed with other templates. The value is basically a one big tuple
//! nesting many other `Render` values. `Render` is implemented for many standard types,
//! can be implemented for new types or can be generated using functions/closures.
//!
//! Users are free to use any Rust language primitives to generate their
//! templates and structure relationship between them in any way that
//! suits them.
//!
//! ## Pros
//!
//! * robust: template generation can reuse any existing code and data
//!   structures
//! * convenient: Rust tooling can work with plain-Rust-templates just
//!   like any other code; `rustfmt` takes care of formatting, typos result
//!   in normal error messages etc.
//! * fast: the compiler optimizes the template code to essential logic
//!   necessary to write-out the rendered template data to the IO; there
//!   is no parsing involved
//!
//! ## Cons
//!
//! * `nightly`-only: This library relies on some unstable features:
//!    * `#![feature(unboxed_closures)]`
//!    # `![feature(fn_traits)]`
//! * immature and incomplete: This library is still work in progress, and will
//!   mature with time.
//!
//! # Where to start
//!
//! You are most probably interested in reading `html` module documentation
#![feature(unboxed_closures)]
#![feature(fn_traits)]
use std::fmt::Arguments;
use std::{fmt, io};

/// HTML rendering
pub mod html;

/// Rendering logic responsible for string escaping and such.
///
/// See `html::Renderer` for implementation.
pub trait Renderer {
    /// Normal write: perform escaping etc. if necessary
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.write_raw(data)
    }
    /// Normal write but with `format_args!`
    fn write_fmt(&mut self, fmt: &Arguments) -> io::Result<()> {
        self.write(format!("{}", fmt).as_bytes())
    }
    /// Normal write for `&str`
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.write(s.as_bytes())
    }

    /// Raw write: no escaping should be performed
    fn write_raw(&mut self, data: &[u8]) -> io::Result<()>;

    /// Raw write but with `format_args!`
    fn write_raw_fmt(&mut self, fmt: &Arguments) -> io::Result<()> {
        self.write_raw(format!("{}", fmt).as_bytes())
    }

    /// Raw write for `&str`
    fn write_raw_str(&mut self, s: &str) -> io::Result<()> {
        self.write_raw(s.as_bytes())
    }
}

/// A `Renderer` that does not escape anything it renders
///
/// A `Renderer` that uses underlying Renderer to call
/// only `raw` methods, and thus avoid escaping values.
pub struct RawRenderer<'a, T: 'a + ?Sized>(&'a mut T);

impl<'a, T: 'a + Renderer + ?Sized> Renderer for RawRenderer<'a, T> {
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.0.write_raw(data)
    }
    fn write_fmt(&mut self, fmt: &Arguments) -> io::Result<()> {
        self.0.write_raw_fmt(fmt)
    }
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.0.write_raw_str(s)
    }
    fn write_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.0.write_raw(data)
    }
    fn write_raw_fmt(&mut self, fmt: &Arguments) -> io::Result<()> {
        self.0.write_raw_fmt(fmt)
    }
    fn write_raw_str(&mut self, s: &str) -> io::Result<()> {
        self.0.write_raw_str(s)
    }
}

/// A value that can be rendered - part or a whole template
///
/// This can be generally thought as a part or a whole `template`,
/// with "the blanks" already filled with a data, but not yet
/// rendered to `Renderer`.
///
/// It is defined for bunch of `std` types. Please send PR if
/// something is missing.
///
/// You can impl it for your own types too. You usually compose it
/// from many other `impl Render` data.
pub trait Render {
    fn render(&self, &mut Renderer) -> io::Result<()>;
}

// {{{ impl Render
impl<T: Render> Render for Vec<T> {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        for t in self.iter() {
            t.render(r)?;
        }
        Ok(())
    }
}

impl<T: Render> Render for [T] {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        for t in self.iter() {
            t.render(r)?;
        }
        Ok(())
    }
}

macro_rules! impl_narr {
    ($n:expr) => {
        impl<T: Render> Render for [T; $n] {
            fn render(&self, r: &mut Renderer) -> io::Result<()> {
                for t in self.iter() {
                    t.render(r)?;
                }
                Ok(())
            }
        }
    };
}

impl_narr!(0);
impl_narr!(1);
impl_narr!(2);
impl_narr!(3);
impl_narr!(4);
impl_narr!(5);
impl_narr!(6);
impl_narr!(7);
impl_narr!(8);
impl_narr!(9);
impl_narr!(10);
impl_narr!(11);
impl_narr!(12);
impl_narr!(13);
impl_narr!(14);
impl_narr!(15);
impl_narr!(16);
impl_narr!(17);
impl_narr!(18);
impl_narr!(19);
impl_narr!(20);
impl_narr!(21);
impl_narr!(22);
impl_narr!(23);
impl_narr!(24);
impl_narr!(25);
impl_narr!(26);
impl_narr!(27);
impl_narr!(28);
impl_narr!(29);
impl_narr!(30);
impl_narr!(31);
impl_narr!(32);

impl<'a, T: Render + ?Sized> Render for &'a mut T {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        (**self).render(r)?;
        Ok(())
    }
}

impl<T: Render + ?Sized> Render for Box<T> {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        (**self).render(r)?;
        Ok(())
    }
}

impl Render for () {
    fn render(&self, _: &mut Renderer) -> io::Result<()> {
        Ok(())
    }
}

impl<R: Render> Render for Option<R> {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        if let &Some(ref s) = self {
            s.render(r)?
        }
        Ok(())
    }
}
impl Render for String {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        r.write_raw(self.as_bytes())
    }
}

macro_rules! impl_render_raw {
    ($t:ty) => {
        impl Render for $t {
            fn render(&self, r: &mut Renderer) -> io::Result<()> {
                r.write_raw_fmt(&format_args!("{}", self))
            }
        }
    };
}

impl_render_raw!(f64);
impl_render_raw!(f32);
impl_render_raw!(i64);
impl_render_raw!(u64);
impl_render_raw!(i32);
impl_render_raw!(u32);
impl_render_raw!(usize);
impl_render_raw!(isize);

impl<'a> Render for &'a str {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        r.write_str(self)
    }
}

impl<'a> Render for fmt::Arguments<'a> {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        r.write_fmt(self)
    }
}

impl<'a> Render for &'a fmt::Arguments<'a> {
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        r.write_fmt(self)
    }
}

impl<A> Render for (A,)
where
    A: Render,
{
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)
    }
}

impl<A, B> Render for (A, B)
where
    A: Render,
    B: Render,
{
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
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
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
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
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)?;
        self.1.render(r)?;
        self.2.render(r)?;
        self.3.render(r)
    }
}
impl<A, B, C, D, E> Render for (A, B, C, D, E)
where
    A: Render,
    B: Render,
    C: Render,
    D: Render,
    E: Render,
{
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)?;
        self.1.render(r)?;
        self.2.render(r)?;
        self.3.render(r)?;
        self.4.render(r)
    }
}

impl<A, B, C, D, E, F> Render for (A, B, C, D, E, F)
where
    A: Render,
    B: Render,
    C: Render,
    D: Render,
    E: Render,
    F: Render,
{
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)?;
        self.1.render(r)?;
        self.2.render(r)?;
        self.3.render(r)?;
        self.4.render(r)?;
        self.5.render(r)
    }
}

impl<A, B, C, D, E, F, G> Render for (A, B, C, D, E, F, G)
where
    A: Render,
    B: Render,
    C: Render,
    D: Render,
    E: Render,
    F: Render,
    G: Render,
{
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)?;
        self.1.render(r)?;
        self.2.render(r)?;
        self.3.render(r)?;
        self.4.render(r)?;
        self.5.render(r)?;
        self.6.render(r)
    }
}

impl<A, B, C, D, E, F, G, H> Render for (A, B, C, D, E, F, G, H)
where
    A: Render,
    B: Render,
    C: Render,
    D: Render,
    E: Render,
    F: Render,
    G: Render,
    H: Render,
{
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        self.0.render(r)?;
        self.1.render(r)?;
        self.2.render(r)?;
        self.3.render(r)?;
        self.4.render(r)?;
        self.5.render(r)?;
        self.6.render(r)?;
        self.7.render(r)?;
        Ok(())
    }
}

/// Use to wrap closures with
pub struct Fn<F>(pub F);

impl<F> Render for Fn<F>
where
    F: std::ops::Fn(&mut Renderer) -> io::Result<()>,
{
    fn render(&self, r: &mut Renderer) -> io::Result<()> {
        self.0(r)
    }
}
// }}}
//
// vim: foldmethod=marker foldmarker={{{,}}}
