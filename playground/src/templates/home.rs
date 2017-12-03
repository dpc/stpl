use stpl::Render;

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub name: String,
}

pub fn page<'a>(data: &'a Data) -> impl Render + 'a {
    use stpl::html::*;
    html(body((
        h1.class("main")("Welcome!"),
        p(format!("Hi, {}", data.name)),
    )))
}

