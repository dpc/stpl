use stpl::Render;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub name: String,
}

pub fn page(data: Data) -> impl Render {
    use stpl::html::*;
    html(body((
        h1.class("main")("Welcome!"),
        p(format!("Hi, {}", data.name)),
    )))
}

