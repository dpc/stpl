#![feature(universal_impl_trait)]
#![feature(conservative_impl_trait)]

extern crate stpl;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::io::Write;
use stpl::html;

pub mod templates;
use templates::*;


pub fn print_template(tpl: impl stpl::Render) {
    let mut v = vec![];
    tpl
        .render(&mut stpl::html::Renderer::new(&mut v))
        .unwrap();
    std::io::stdout().write_all(&v).unwrap();
}

pub fn home_tpl() -> impl stpl::Template {
    html::Template::new("home", ::templates::home::page)
}

fn main() {
    stpl::handle_dynamic()
        .template(&home_tpl());

    let data = templates::home::Data {
        page: base::Data {
            title: "Hello!".into(),
        },
        name: "William".into()
    };

    println!("Change `src/templates/home.rs` and rerun `cargo build` to pick a new template version");
    println!();
    loop {
        println!("Static:");
        print_template(templates::home::page(&data));
        println!("");
        println!("dynamic:");
        std::io::stdout().write_all(&stpl::render_dynamic_self(&home_tpl(), &data).unwrap()).unwrap();
        println!("");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

}
