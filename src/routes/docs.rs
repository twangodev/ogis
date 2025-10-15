use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::index::generate,
        crate::routes::health::health_check
    ),
    components(schemas(crate::routes::index::OgParams)),
    info(
        title = "OGIS - Open Graph Image as a Service",
        version = "0.1.0",
        description = "Generate Open Graph images dynamically via URL parameters"
    ),
    tags(
        (name = "image", description = "Image generation endpoints"),
        (name = "monitoring", description = "Service monitoring and health checks")
    )
)]
pub struct ApiDoc;
