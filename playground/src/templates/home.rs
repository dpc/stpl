use stpl::Render;

use super::base;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub page: base::Data,
    pub name: String,
}

pub fn page(data: &Data) -> impl Render {
    use stpl::html::*;
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let content = (
        h1.class("main")("Welcome page"),
        p(
            format!("Hi, {}!", data.name)
        ),
        ul(
            (0..2).map(|n| li(n)).collect::<Vec<_>>()
        )
    );

    base::base(&data.page, content)
}
