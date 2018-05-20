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
//! base-skeleton template for a Bootstrap-based UI.
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
//! It is just a function. There is no magic, no macros, no textfiles involved.
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
//! ### Dynamic rendering
//!
//! While `stpl` generates Rust code and does not invole "runtime parsing",
//! it supports doing the actual rendering in a
//! separate process, thus hot-swapping the templates at runtime. This is very
//! useful for speeding up development.
//!
//! The basic mechanism is:
//!
//! * serialize the template data, and send it to a child process
//! * read the rendered template back from the child
//!
//! In the child process:
//!
//! * identify the template to use,
//! * read the serialized data from the stdio and deserialize it
//! * render the template and output it to stdout
//!
//! In this scheme the binary for parent and child processes can be the same
//! (see `render_dynamic_self`) or different (see `render_dynamic).
//!
//! Using the same binary is more convenient. Using separate binaries
//! requires structuring the project in a certain way, but can greatly
//! improve iteration time.
//!
//! The following is an exerpt from `Cargo.toml` to support dynamic
//! rendering in a separate binary:
//!
//! ```norust
//!
//! [[bin]]
//! name = "template"
//! path = "src/main_template.rs"
//!
//! [[bin]]
//! name = "webapp"
//! path = "src/main.rs"
//! ```
//!
//! These two programs share many modules (eg. templates and data structures),
//! but `main_template` does not have to include any heavy-duty libraries like
//! `rocket`, `diesel` and similar, thus compiles much faster.
//!
//! In our tests it takes 11.4 secs to build the main webapp in debug mode,
//! while recompiling all templates is much faster:
//!
//! ```norust
//! $ cargo build --bin template
//! Compiling webapp v0.1.0 (file:///home/dpc/lab/rust/webapp/web)
//! Finished dev [unoptimized + debuginfo] target(s) in 1.04 secs
//! ```
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
//! * fast iteration: with dynamic loading it's possible to reload templates
//!   without waiting for Rust compiler to build the whole app
//!
//! ## Cons
//!
//! * `nightly`-only: This library relies on some unstable features (mostly
//!   `impl trait`)
//! * immature and incomplete: This library is still work in progress, and will
//!   mature with time.
//!
//! # Where to start
//!
//! You are most probably interested in reading `html` module documentation
//!
//! # Help
//!
//! Please see `./playground` subdirectory for example usage.
#![feature(unboxed_closures)]
#![feature(fn_traits)]
extern crate bincode;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate serde;

use std::{fmt, io};
use std::fmt::Arguments;
use std::io::Read;
use std::path::Path;

/// HTML rendering
pub mod html;

lazy_static! {
    static ref IS_ENV_STPL_PROD: bool = {
        if let Ok(_val) = std::env::var("STPL_PROD") {
            true
        } else {
            false
        }
    };
}

/// A whole template that knows how to render itself
///
/// See `html::Template` for the actual type implementing it
/// and more concrete information.
pub trait Template {
    type Argument: serde::Serialize + for<'de> serde::Deserialize<'de>;
    /// A unique key used to identify the template
    fn key(&self) -> &'static str;

    /// Render itself into an `io`
    fn render<'a>(&self, argument: &Self::Argument, io: &'a mut io::Write) -> io::Result<()>;
}

/// Convinience methods for `Template`
pub trait TemplateExt: Template {
    /// Call current binary file to handle the template
    ///
    /// Make sure to put `handle_dynamic` at the very beginning of your
    /// binary if you want to use it.
    ///
    /// See `render_dynamic` for more info.
    ///
    /// This function will behave like `render_static` if
    /// `STPL_PROD` environment variable is enabled.
    fn render_dynamic_self(&self, data: &<Self as Template>::Argument) -> DynamicResult<Vec<u8>>
    where
        Self: Sized,
        <Self as Template>::Argument: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
    {
        if *IS_ENV_STPL_PROD {
            self.render_static(data).map_err(|e| e.into())
        } else {
            render_dynamic_self(self, data)
        }
    }

    /// Call a template dynamically (with ability to update at runtime)
    ///
    /// Make sure to put `handle_dynamic` at the very beginning of the code
    /// of program under `path`.
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
    /// This function will behave like `render_static` if
    /// `STPL_PROD` environment variable is enabled.
    fn render_dynamic(
        &self,
        path: &Path,
        data: &<Self as Template>::Argument,
    ) -> DynamicResult<Vec<u8>>
    where
        Self: Sized,
        <Self as Template>::Argument: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
    {
        if *IS_ENV_STPL_PROD {
            self.render_static(data).map_err(|e| e.into())
        } else {
            render_dynamic(path, self, data)
        }
    }

    fn render_static(&self, data: &<Self as Template>::Argument) -> io::Result<Vec<u8>>
    where
        Self: Sized,
        <Self as Template>::Argument: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
    {
        let mut v = vec![];
        self.render(data, &mut v)?;
        Ok(v)
    }
}

impl<T: Template> TemplateExt for T {}
/// Rendering logic responsible for string escaping and such.
///
/// See `html::Renderer` for implementation.
pub trait Renderer {
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.write_raw(data)
    }
    fn write_fmt(&mut self, fmt: &Arguments) -> io::Result<()> {
        self.write(format!("{}", fmt).as_bytes())
    }
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.write(s.as_bytes())
    }
    fn write_raw(&mut self, data: &[u8]) -> io::Result<()>;

    fn write_raw_fmt(&mut self, fmt: &Arguments) -> io::Result<()> {
        self.write_raw(format!("{}", fmt).as_bytes())
    }
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
    }
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
    }
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

fn handle_dynamic_impl<T: Template + ?Sized>(template: &T) -> io::Result<()> {
    let mut v = vec![];
    std::io::stdin().read_to_end(&mut v)?;
    let arg: T::Argument = bincode::deserialize(&v[..])
        .map_err(|_e| io::Error::new(io::ErrorKind::Other, "Deserialization error"))?;

    let stdout = std::io::stdout();
    let stdout = stdout.lock();
    let mut out = std::io::BufWriter::new(stdout);
    template.render(&arg, &mut out)?;

    Ok(())
}

/// `handle_dynamic` handle
pub struct HandleDynamic;

pub fn handle_dynamic() -> HandleDynamic {
    HandleDynamic
}

/// Exit code used by the dynamic template binary on success
///
/// This code is non-zero to make it different from typical
/// success exit code of an unsuspecting binary
pub const EXIT_CODE_SUCCESS: i32 = 66;

/// Exit code used by the dynamic template binary on failure
///
/// The stderr of the process will contain more information.
pub const EXIT_CODE_FAILED: i32 = 67;

/// Exit code used by the deyamic template binary when template key was not found
pub const EXIT_CODE_NOT_FOUND: i32 = 68;

const ENV_NAME: &'static str = "RUST_STPL_DYNAMIC_TEMPLATE_KEY";

impl HandleDynamic {
    pub fn template<A: serde::Serialize + for<'de> serde::Deserialize<'de>>(
        self,
        template: &Template<Argument = A>,
    ) -> HandleDynamic {
        // TODO: optimize, don't fetch every time?
        if let Ok(var_name) = std::env::var(ENV_NAME) {
            if var_name.as_str() == template.key() {
                match handle_dynamic_impl(template) {
                    Ok(_) => std::process::exit(EXIT_CODE_SUCCESS),
                    Err(e) => {
                        eprintln!("Dynamic template process failed: {:?}", e);
                        std::process::exit(EXIT_CODE_FAILED);
                    }
                }
            }
        }

        self
    }
}

impl std::ops::Drop for HandleDynamic {
    fn drop(&mut self) {
        if let Ok(var_key) = std::env::var(ENV_NAME) {
            if !var_key.is_empty() {
                eprintln!("Couldn't find dynamic template by key: {}", var_key);
                std::process::exit(EXIT_CODE_NOT_FOUND);
            }
        }
    }
}

#[derive(Fail, Debug)]
pub enum DynamicError {
    #[fail(display = "IO error")] Io(#[cause] io::Error),
    #[fail(display = "Template not found: {}", key)] NotFound {
        key: String,
    },
    #[fail(display = "Template failed")]
    Failed {
        exit_code: Option<i32>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
}

impl From<io::Error> for DynamicError {
    fn from(e: io::Error) -> Self {
        DynamicError::Io(e)
    }
}

type DynamicResult<T> = std::result::Result<T, DynamicError>;

fn render_dynamic<'a, 'path, A: 'static, T: Template>(
    path: &'path Path,
    template: &'a T,
    data: &'a A,
) -> DynamicResult<Vec<u8>>
where
    A: serde::Serialize,
{
    let encoded: Vec<u8> = bincode::serialize(&data, bincode::Infinite).unwrap();

    use std::process::{Command, Stdio};
    use std::io::Write;

    let mut child = Command::new(path)
        .env(ENV_NAME, template.key())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute child");

    // TODO: Sending and receiving could be done in a separate thread too, and
    // some form of a "future" could be used
    {
        let stdin = child.stdin.as_mut().expect("failed to get stdin");
        stdin.write_all(&encoded).expect("failed to write to stdin");
        stdin.flush().expect("failed to flush stdin");
    }
    child.stdin = None;

    let out = child.wait_with_output()?;
    match out.status.code() {
        Some(EXIT_CODE_SUCCESS) => Ok(out.stdout),
        Some(EXIT_CODE_NOT_FOUND) => Err(DynamicError::NotFound {
            key: template.key().to_owned(),
        }),
        code => Err(DynamicError::Failed {
            exit_code: code,
            stdout: out.stdout,
            stderr: out.stderr,
        }),
    }
}

fn render_dynamic_self<T: Template>(
    template: &T,
    data: &<T as Template>::Argument,
) -> DynamicResult<Vec<u8>>
where
    <T as Template>::Argument: serde::Serialize + 'static,
{
    let path = std::env::args_os().next().unwrap();
    let path = path.as_ref();
    render_dynamic(path, template, data)
}

// vim: foldmethod=marker foldmarker={{{,}}}
