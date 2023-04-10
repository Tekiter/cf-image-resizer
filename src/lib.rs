use std::{collections::HashMap, io::Cursor};

use futures::stream::StreamExt;
use image::{imageops::FilterType, io::Reader as ImageReader, ImageOutputFormat};
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
        .get("/", |_, _| Response::ok("Hello from Workers!"))
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
        .get_async("/image", |mut req, ctx| async move {
            let url = req.url().unwrap();
            let query: HashMap<_, _> = url.query_pairs().collect();

            let url = query.get("url").expect("url 가져오기 실패");
            let w = query.get("w").expect("no w").parse::<u32>().unwrap();

            let fetcher = Fetch::Url(Url::parse(url).expect("url 파스 실패"));

            let mut res = fetcher.send().await.expect("이미지 가져오기 실패");
            let buffer = res.stream().expect("스트림 가져오기 실패");
            let bytes = buffer
                .map(|entry| entry.expect("buffer map 실패"))
                .concat()
                .await;

            let reader = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .expect("This will never fail using Cursor");

            let img = reader.decode().expect("should decoded");

            let resized = img.resize(w, w, FilterType::Nearest);

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
