mod client;
mod content;
mod html;
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

use std::{string::ToString, time::Instant};

use client::{Client, PostDataClient};
use content::Render;
use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT: Client = Client;
}

fn render_post(post_id: &str) -> String {
    let time_start = Instant::now();
    let post = CLIENT.get_post_data(post_id).unwrap().get_post();
    let duration = time_start.elapsed();
    println!("fetching {} took {}", post_id, duration.as_secs_f32());
    html::html_page(&post.title, &post.render().unwrap().to_string())
}

#[fn_handler]
#[cfg(feature = "standalone")]
async fn handle_response_standalone(req: &mut salvo::Request, res: &mut salvo::Response) {
    if req.uri().to_string().contains("favicon") {
        return;
    }

    let content = match req.params().get("postid") {
        Some(postid) => render_post(postid),
        None => html::home(),
    };
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
    let content = match params.first("postid") {
        Some(postid) => render_post(postid),
        None => html::home(),
    };
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
        let router = Router::new()
            .push(Router::with_path("<postid>").get(handle_response_standalone))
            .push(Router::new().get(handle_response_standalone));
        Server::new(TcpListener::bind("127.0.0.1:7878"))
            .serve(router)
            .await;
    }

    Ok(())
}
