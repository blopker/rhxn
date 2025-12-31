use crate::types;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub stories: Vec<types::Item>,
    pub title: &'a str,
    pub static_base: &'a str,
}

#[derive(Template)]
#[template(path = "item.html")]
pub struct ItemTemplate<'a> {
    pub item: types::Item,
    pub title: String,
    pub static_base: &'a str,
    pub comments: Option<String>,
}

#[derive(Template)]
#[template(path = "comment.html")]
pub struct CommentTemplate {
    pub item: types::Item,
    pub comments: Option<String>,
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
