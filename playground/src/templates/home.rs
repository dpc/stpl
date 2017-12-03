use stpl::Render;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub name: String,
}

pub fn page(data: Data) -> impl Render {
    use stpl::html::*;
    #[cfg_attr(rustfmt, rustfmt_skip)]
    html(
        body(
            (
                h1.class("main")("Welcome page"),
                p(format!("Hi, {}!", data.name)),
            )
        )
    )
}
