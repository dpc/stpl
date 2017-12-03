use std::io;

use {Render, Renderer};


/// Implement a Render function wrapping in a simple tag
macro_rules! impl_tag {
    ($t:ident) => {
        pub fn $t(inner: impl Render) -> impl Render {
            wrap_in_tag(stringify!($t), inner)
        }
    };
}

impl_tag!(li);
impl_tag!(ul);
impl_tag!(ol);
impl_tag!(p);

fn wrap_in_tag(tag : &'static str, inner: impl Render) -> impl Render {
    move |r : &mut Renderer| -> io::Result<()> {
        r.write_raw_str("<")?;
        r.write_raw_str(tag)?;
        r.write_raw_str(">")?;
        inner.render(r)?;
        r.write_raw_str("</")?;
        r.write_raw_str(tag)?;
        r.write_raw_str(">")
    }
}

fn render_list(list: Vec<&str>) -> impl Render {

    ul(
        list.iter().enumerate().map(|(i, _s)| p(i)).collect::<Vec<_>>()
    )
}
