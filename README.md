<!-- README.md is auto-generated from README.tpl with `cargo readme` -->

<p align="center">
  <a href="https://travis-ci.org/dpc/stpl">
      <img src="https://img.shields.io/travis/dpc/stpl/master.svg?style=flat-square" alt="Travis CI Build Status">
  </a>
  <a href="https://crates.io/crates/stpl">
      <img src="http://meritbadge.herokuapp.com/stpl?style=flat-square" alt="crates.io">
  </a>
  <a href="https://gitter.im/dpc/stpl">
      <img src="https://img.shields.io/badge/GITTER-join%20chat-green.svg?style=flat-square" alt="Gitter Chat">
  </a>
  <br>
</p>

# stpl

### stpl - Super template library for Rust

`stpl` is plain-Rust template library.

### Main idea

In `stpl` there are no magic macros or DSLs, and no clunky
text-files with weird syntax. Everything is just normal, easy
to understand Rust code.


Let's take a look at a real-life example from the pilot project:

```rust,no_run
pub fn base<C: Render + 'static>(data: &Data, content: C) -> impl Render {
    (
        doctype("html"),
        html((
            head((
                meta.charset("utf-8"),
                meta.name("viewport").content("width=device-width, initial-scale=1, shrink-to-fit=no"),
                meta.name("description").content(""),
                meta.name("author").content("Dawid Ciężarkiewicz"),
                title(data.title.clone()),
                (
                    link.rel("icon").href("/static/favicon.ico"),
                    link.rel("stylesheet").href("/static/theme/flatly/bootstrap.min.css"),
                    link.rel("stylesheet").href("/static/theme/starter-template.css"),
                )
            )),
            body((
                navbar(data),
                main
                    .id("main")
                    .role("main")
                    .class("container mb-5")(
                    content,
                ),
                (
                script.src("https://code.jquery.com/jquery-3.2.1.min.js").crossorigin("anonymous"),
                script.src("https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.3/umd/popper.min.js")
                    .integrity("sha384-vFJXuSJphROIrBnz7yo7oB41mKfc8JzQZiCq4NCceLEaO4IHwicKwpJf9c9IpFgh")
                    .crossorigin("anonymous"),
                script.src("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/js/bootstrap.min.js")
                    .integrity("sha384-alpBpkh1PFOepccYVYDB4do5UnbKysX5WZXm3XxPqe5iKTfUKjNkCk9SaVuEZflJ")
                    .crossorigin("anonymous"),
                script.type_("text/javascript")(
                    raw(include_str!("white-icon.js"))
                ),
                )
            ))
        ))
    )
}
```rust

The above is an HTML base-template. It contains base HTML skeleton of a
Bootstrap-based UI.

It is just a function. There is no magic, no macros, no textfiles involved.
The whole template was formatted with `rustfmt` just like a normal Rust code.

The function accepts arguments:

* `data: Data` containing information how to "fill the blanks", and
* `content: Render` - sub-template value that will be used as main page content.

The function returns `Render` value that can be rendered as a string or bytes, or
composed with other templates. The value is basically a one big tuple
nesting many other `Render` values. `Render` is implemented for many standard types,
can be implemented for new types or can be generated using functions/closures.

Users are free to use any Rust language primitives to generate their
templates and structure relationship between them in any way that does
suits them.

### Dynamic rendering

While `stpl` generates Rust code and does not invole "runtime parsing",
it supports doing the actual rendering in a
separate process, thus hot-swapping the templates at runtime. This is very
useful for speeding up development.

The basic mechanism is:

* serialize the template data, and send it to a child process
* read the rendered template back from the child

In the child process:

* identify the template to use,
* read the serialized data from the stdio and deserialize it
* render the template and output it to stdout

In this scheme the binary for parent and child processes can be the same
(see `render_dynamic_self`) or different (see `render_dynamic).

Using the same binary is more convenient. Using separate binaries
requires structuring the project in a certain way, but can greatly
improve iteration time.

The following is an exerpt from `Cargo.toml` to support dynamic
rendering in a separate binary:

```norust

[[bin]]
name = "template"
path = "src/main_template.rs"

[[bin]]
name = "webapp"
path = "src/main.rs"
```

These two programs share many modules (eg. templates and data structures),
but `main_template` does not have to include any heavy-duty libraries like
`rocket`, `diesel` and similar, thus compiles much faster.

In our tests it takes 11.4 secs to build the main webapp in debug mode,
while recompiling all templates is much faster:

```rust
$ cargo build --bin template
Compiling communityaudit v0.1.0 (file:///home/dpc/lab/rust/hackeraudit/web)
Finished dev [unoptimized + debuginfo] target(s) in 1.04 secs
```

### Drawbacks

This library relies on some `nightly`-only unstable features

## Where to start

You are most probably interested in reading `html` module documentation

## Help

Please see `./playground` subdirectory for example usage.

# License

stpl is licensed under: MPL-2.0/MIT/Apache-2.0
