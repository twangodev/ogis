use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::index::generate,
        crate::routes::health::health_check,
        crate::routes::templates::list_templates
    ),
    components(schemas(
        crate::params::OgParams,
        crate::error::ErrorBody,
        crate::error::ErrorDetail,
        crate::error::ErrorCode,
        crate::routes::templates::TemplatesResponse
    )),
    info(
        title = "ogis: Open Graph Images as a Service",
        version = "0.1.0",
        description = "Generate Open Graph images dynamically via URL parameters"
    ),
    servers(
        (url = "https://img.ogis.dev")
    ),
    tags(
        (name = "image", description = "Image generation endpoints"),
        (name = "monitoring", description = "Service monitoring and health checks")
    )
)]
pub struct ApiDoc;
