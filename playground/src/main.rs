#![feature(universal_impl_trait)]
#![feature(conservative_impl_trait)]

extern crate stpl;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::io::Write;
use stpl::Render;

pub mod templates;

pub fn print_template(tpl: impl stpl::Render) {
    let mut v = vec![];
    tpl
        .render(&mut stpl::html::Renderer::new(&mut v))
        .unwrap();
    std::io::stdout().write_all(&v).unwrap();
}

fn main() {
    stpl::handle_dynamic("home", templates::home::page);

    let data = templates::home::Data {
        name: "William".into()
    };

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Static:");
        print_template(templates::home::page(&data));
        println!("");
        println!("dynamic:");
        //stpl::call_dynamic("home", templates::home::page);
        println!("");
    }

}
