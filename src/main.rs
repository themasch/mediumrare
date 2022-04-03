mod client;
mod content;
mod text_markup;

#[cfg(all(feature = "lambda", feature = "standalone"))]
compile_error!("can only use either the lambda, or the standalone feature");

#[cfg(not(feature = "standalone"))]
use lambda_http::{
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    service_fn, Error, IntoResponse, Request, RequestExt, Response,
};

#[cfg(feature = "standalone")]
use salvo::{
    async_trait, fn_handler,
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    prelude::TcpListener,
    Router, Server,
};

use std::string::ToString;

use client::{Client, PostDataClient};
use content::{Page, Render};
use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT: Client = Client;
}

fn render_post(post_id: &str) -> String {
    let page = Page::create(CLIENT.get_post_data(post_id).unwrap().get_post());
    page.render().to_string()
}

#[fn_handler]
#[cfg(feature = "standalone")]
async fn handle_response_standalone(req: &mut salvo::Request, res: &mut salvo::Response) {
    if req.uri().to_string().contains("favicon") {
        return;
    }

    let postid = req.params().get("postid").unwrap();
    let content = render_post(postid);
    res.set_status_code(StatusCode::OK);
    res.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    res.write_body_bytes(content.as_bytes());
}

#[cfg(not(feature = "standalone"))]
async fn handle_response_aws(event: Request) -> Result<impl IntoResponse, Error> {
    let params = event.path_parameters();
    let postid = params.first("postid").unwrap_or("633ff591866e");
    let content = render_post(postid);
    let builder = Response::builder()
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        )
        .status(StatusCode::OK);

    Ok(builder.body(content).expect("failed to build response"))
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    #[cfg(not(feature = "standalone"))]
    lambda_http::run(service_fn(handle_response_aws))
        .await
        .map_err(|_| ())?;

    #[cfg(feature = "standalone")]
    {
        let router = Router::with_path("/<postid>").get(handle_response_standalone);
        Server::new(TcpListener::bind("127.0.0.1:7878"))
            .serve(router)
            .await;
    }

    Ok(())
}
