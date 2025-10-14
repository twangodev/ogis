use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(crate::routes::index::handler),
    components(schemas(crate::routes::index::OgParams)),
    info(
        title = "OGIS - Open Graph Image Service",
        version = "0.1.0",
        description = "Generate Open Graph images dynamically via URL parameters"
    )
)]
pub struct ApiDoc;
