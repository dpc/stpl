#![feature(universal_impl_trait)]
#![feature(conservative_impl_trait)]

extern crate stpl;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod templates;

fn main() {
    let data = templates::home::Data {
        name: "William".into()
    };

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Static:");
        templates::home::print_static(&data);
        println!("dynamic:");
        templates::home::print_dynamic(&data);
    }

}
