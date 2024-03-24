use axum::{
    extract::{DefaultBodyLimit, Multipart},
    response::Html,
    routing::get,
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;
use tracing_subscriber::{layer::SubscriberExt};
use image::{imageops::FilterType, io::Reader as ImageReader, DynamicImage, ImageFormat};
pub use self::multipart::Multipart;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_multipart_form=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with some routes
    let app = Router::new()
        // post doesn't work on errors
        // my code returns an error
        .route("/", post(show_form))
        .route("/upload", post(upload_image))
        .route("/convert", post(convert_to))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            250 * 1024 * 1024, /* 250mb */
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <label>
                        Upload file:
                        <input type="file" name="file" multiple>
                    </label>

                    <input type="submit" value="Upload files">
                </form>
            </body>
        </html>
        "#,
    )
}

// upload the image
//
// convert it
//
//
async fn upload_image(mut multipart: Multipart) -> Result<Html<String>, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let content_type = field.content_type();
        let bytes = field.bytes().await.unwrap()?;
        let image_data = bytes.to_vec();

        let mut headers = HeaderMap::new();

        convert_to(image_data, content_type, &mut headers).expect("File missing or too big");

        return Ok(format!(
                "Image successfully converted"
        ));

        Err(StatusCode::BAD_REQUEST)

            }
}

async fn convert_to(content_type: &str, image_data: Vec<u8>, headermap: &HeaderMap) -> Result<Response<Vec<u8>>, image::ImageError> {
    match content_type {
        "image/png" => {
            let mut png_data = Vec::new();

            image_data.write_to(png_data, ImageFormat::Png)?;

            headers.insert(content_type, HeaderValue::from_static("image/png"));
            Ok(Response::new(png_data))
        }

        "image/jpeg" => {
            let mut jpeg_data = Vec::new();
            image_data.write_to(jpeg_data, ImageFormat::Jpeg)?;

            headers.insert(content_type, HeaderValue::from_static("image/jpeg"));
            Ok(Response::new(jpeg_data))
        }
        _            => {
            eprintln!("File type not supported");
        }
    }
}
