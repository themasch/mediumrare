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

#[derive(Debug, thiserror::Error)]
enum LocalError {
    #[error("client error: {0:?}")]
    ClientError(#[from] client::ClientError),
}

fn render_post(post_id: &str) -> Result<String, LocalError> {
    let time_start = Instant::now();
    let post = CLIENT.get_post_data(post_id)?.get_post();
    let duration = time_start.elapsed();
    println!("fetching {} took {}", post_id, duration.as_secs_f32());
    Ok(html::html_page(
        &post.title,
        &post.render().unwrap().to_string(),
    ))
}

fn map_error(res: Result<String, LocalError>) -> (StatusCode, String) {
    match res {
        Ok(c) => (StatusCode::OK, c),
        Err(LocalError::ClientError(err)) => (StatusCode::NOT_FOUND, err.to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

#[fn_handler]
#[cfg(feature = "standalone")]
async fn handle_response_standalone(req: &mut salvo::Request, res: &mut salvo::Response) {
    if req.uri().to_string().contains("favicon") {
        return;
    }

    let (status_code, content) = map_error(match req.params().get("postid") {
        Some(postid) if postid.len() >= 1 => render_post(postid),
        Some(_) => Ok(html::home()),
        None => Ok(html::home()),
    });

    res.set_status_code(status_code);
    res.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    res.write_body_bytes(content.as_bytes());
}

#[cfg(not(feature = "standalone"))]
async fn handle_response_aws(event: Request) -> Result<impl IntoResponse, Error> {
    let params = event.path_parameters();
    let (status_code, content) = map_error(match params.first("postid") {
        Some(postid) if postid.len() >= 1 => render_post(postid),
        Some(_) => Ok(html::home()),
        None => Ok(html::home()),
    });

    let builder = Response::builder()
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        )
        .status(status_code);

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
