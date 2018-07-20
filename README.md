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

`stpl` is a plain-Rust-only template library with some neat properties and
features.

### Main idea

In `stpl` there are no magic macros or DSLs, and no clunky
text-files with weird syntax. Everything is just normal, easy
to understand Rust code.

Let's take a look at a real-life example from the pilot project: an HTML
based-skeleton template for a Bootstrap-based UI.

```rust
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
```

It is just a function. There is no magic, no macros, no text files involved.
The whole template was formatted with `rustfmt` just like a normal Rust code.

The function accepts arguments:

* `data: Data` containing information how to "fill the blanks", and
* `content: Render` - sub-template value that will be used as main page content.

The function returns `Render` value that can be rendered as a string or bytes, or
composed with other templates. The value is basically a one big tuple
nesting many other `Render` values. `Render` is implemented for many standard types,
can be implemented for new types or can be generated using functions/closures.

Users are free to use any Rust language primitives to generate their
templates and structure relationship between them in any way that
suits them.

### Pros

* robust: template generation can reuse any existing code and data
  structures
* convenient: Rust tooling can work with plain-Rust-templates just
  like any other code; `rustfmt` takes care of formatting, typos result
  in normal error messages etc.
* fast: the compiler optimizes the template code to essential logic
  necessary to write-out the rendered template data to the IO; there
  is no parsing involved

### Cons

* `nightly`-only: This library relies on some unstable features:
   * `#![feature(unboxed_closures)]`
   # `![feature(fn_traits)]`
* immature and incomplete: This library is still work in progress, and will
  mature with time.

## Where to start

You are most probably interested in reading `html` module documentation

# License

stpl is licensed under: MPL-2.0/MIT/Apache-2.0
