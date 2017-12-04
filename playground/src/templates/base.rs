use stpl::Render;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub title: String,
}

pub fn base<C : Render+'static>(data: &Data, content : C) -> impl Render  {
    use stpl::html::*;
    #[cfg_attr(rustfmt, rustfmt_skip)]
    html((
        head(
            title(data.title.clone())
        ),
        body(
            content
        )
    ))
}
