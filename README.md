# Cloudflare Workers Image Resize Proxy

An image resize proxy built with Rust and designed to work with Cloudflare Workers.

It provides a way to resize images on the fly, making it useful for web apps that require image optimazation.

## Usage

To use this image resize proxy, simply construct a URL in the following format:

```
https://<your-worker-url>/image?url=<image_url>&w=<resized_width_in_pixel>
```

Replace <your-worker-url> with the URL of your Cloudflare Worker. `<image_url>` should be replaced with the URL of the image you want to resize, and `<resized_width_in_pixel>` should be replaced with the desired width of the resized image, in pixels.

For example, to resize an image located at https://loremflickr.com/1000 to a width of 200 pixels, you would construct the following URL:

```
https://<your-worker-url>/image?url=https://loremflickr.com/1000&w=200
```

## License

This project is licensed under the [MIT License](LICENSE).
