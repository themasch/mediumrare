mod client;
mod content;

use lambda_http::{
    http::StatusCode, service_fn, Error, IntoResponse, Request, RequestExt, Response,
};
use std::string::ToString;

use client::{Client, PostDataClient};
use content::{Page, Render};
use lazy_static::lazy_static;

#[cfg(not(test))]
lazy_static! {
    static ref CLIENT: Client = Client;
}

#[cfg(test)]
mod mock_client;
#[cfg(test)]
use crate::mock_client::MockClient;
#[cfg(test)]
lazy_static! {
    static ref CLIENT: MockClient = MockClient;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(event: Request) -> Result<impl IntoResponse, Error> {
    let params = event.path_parameters();

    let postid = params.first("postid").unwrap_or("633ff591866e");

    let page = Page::create(CLIENT.get_post_data(postid).unwrap().get_post());

    let builder = Response::builder()
        .header("Content-Type", "text/html;charset=utf-8")
        .status(StatusCode::OK);

    Ok(builder
        .body(page.render().to_string())
        .expect("failed to build response"))
}
