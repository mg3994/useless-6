use rust_embed::RustEmbed;
use salvo::prelude::*;
use salvo::serve_static::{static_embed, EmbeddedFileExt};
use typedef::AppResult;


#[derive(RustEmbed)]
#[folder = "../../assets"]
struct Assets;

pub fn root() -> Router {
    let favicon = Assets::get("favicon.ico")
        .expect("favicon not found")
        .into_handler();
    let router = Router::new()
        .hoop(Logger::new())
        .get(hello)
        // ....
        .push(Router::with_path("error").get(dummy_error))
        .push(Router::with_path("favicon.ico").get(favicon))
        .push(Router::with_path("assets/{**rest}").get(static_embed::<Assets>()));
    let doc = OpenApi::new("salvo web api", "0.0.1").merge_router(&router);
    router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(Scalar::new("/api-doc/openapi.json").into_router("scalar"))
}


/// This is a dummy hello handler that shows up in Scalar
#[endpoint(
    tags("General"),
    responses(
        (status_code = 200, description = "Hello world success", body = String)
    )
)]
async fn hello(res: &mut Response) {
    res.render(Text::Plain("Hello World"));
}

#[endpoint(tags("General"))]
pub async fn dummy_error() -> AppResult<&'static str> {
    Err(typedef::AppError::HttpStatus(salvo::prelude::StatusError::forbidden()))
}