use stpl::Render;
use stpl;
use std;
use std::io::Write;

#[derive(Serialize)]
pub struct Data {
    pub name: String,
}

fn page<'a>(data: &'a Data) -> impl Render + 'a {
    use stpl::html::*;
    html(
        body(
            (
                h1("Welcome!"),
                p(format!("Hi, {}", data.name))
            )
            )
        )
}

fn page<'a>(data: &'a Data) -> impl Render + 'a {
    use stpl::html::*;
    html {
        body.class("foo, bar")(
            (
                h1("Welcome!"),
                p(format!("Hi, {}", data.name))
            )
            )
            )
    }
}
pub fn print_static(data: &Data) {
    let mut v= vec![];
    page(data).render(&mut stpl::html::Renderer::new(&mut v)).unwrap();
    std::io::stdout().write_all(&v).unwrap();
}


pub fn print_dynamic(data: &Data) {
    let mut v= vec![];
    page(data).render(&mut stpl::html::Renderer::new(&mut v)).unwrap();
    std::io::stdout().write_all(&v).unwrap();
}
