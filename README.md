# Cloudflare Workers Image Resize Proxy

- Image Resize Proxy for [Cloudflare Workers](https://workers.cloudflare.com/)
- Written in Rust ([workers-rs](https://github.com/cloudflare/workers-rs))

## Usage

```
https://<your-worker-url>/?url=<image_url>&w=<resized_width_in_pixel>
```

**Example URL**

```
https://localhost:8787/?url=https://loremflickr.com/1000&w=200
```
