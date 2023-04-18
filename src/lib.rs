use std::{collections::HashMap, io::Cursor};

use futures::stream::StreamExt;
use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage, ImageOutputFormat};
use serde_json::json;
use worker::*;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("Hello Image Resizer!"))
        .post_async("/form/:field", |mut req, ctx| async move {
            if let Some(name) = ctx.param("field") {
                let form = req.form_data().await?;
                match form.get(name) {
                    Some(FormEntry::Field(value)) => {
                        return Response::from_json(&json!({ name: value }))
                    }
                    Some(FormEntry::File(_)) => {
                        return Response::error("`field` param in form shouldn't be a File", 422);
                    }
                    None => return Response::error("Bad Request", 400),
                }
            }

            Response::error("Bad Request", 400)
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .get_async("/image", |req, _ctx| async move {
            let url = req.url().unwrap();
            let query: HashMap<_, _> = url.query_pairs().collect();

            let parsed_url = match query.get("url") {
                Some(href) => href,
                None => return Response::error("'url' parameter is not provided.", 403),
            };

            let w = match query.get("w") {
                Some(res) => res.parse::<u32>().unwrap(),
                None => return Response::error("'w' parameter not provided.", 403),
            };

            let bytes = match fetch_image_from_url(parsed_url).await {
                Ok(bytes) => bytes,
                Err(_) => return Response::error("failed to fetch image.", 403),
            };

            let img = read_image(bytes);
            let resized = img.resize(w, img.height(), FilterType::Nearest);

            let outbuf = vec![];
            let mut c = Cursor::new(outbuf);
            resized
                .write_to(&mut c, ImageOutputFormat::Png)
                .expect("write failed");

            let x = c.get_ref().to_vec();

            let mut header = Headers::new();
            header.append("Accept-Ranges", "bytes").unwrap();
            header.append("Content-Type", "image/png").unwrap();
            header
                .append("Content-Length", x.len().to_string().as_ref())
                .unwrap();
            Ok(Response::from_bytes(x)?.with_headers(header))
        })
        .run(req, env)
        .await
}

fn read_image(bytes: Vec<u8>) -> DynamicImage {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("This will never fail using Cursor");

    let img = reader.decode().expect("should decoded");

    img
}

async fn fetch_image_from_url(url: &str) -> std::result::Result<Vec<u8>, ImageFetchError> {
    let url = Url::parse(url).map_err(|_| ImageFetchError::InvalidUrl)?;

    let fetcher = Fetch::Url(url);
    let mut res = fetcher
        .send()
        .await
        .map_err(|_| ImageFetchError::FailedToFetch)?;

    let buffer = res.stream().map_err(|_| ImageFetchError::FailedToFetch)?;

    let bytes = buffer.map(|entry| entry.unwrap_or(vec![])).concat().await;

    Ok(bytes)
}

enum ImageFetchError {
    InvalidUrl,
    FailedToFetch,
}
