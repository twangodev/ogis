use axum::body::Body;
use axum::http::{Request, StatusCode};
use clap::Parser;
use tower::ServiceExt; // oneshot

fn test_state() -> crate::AppState {
    let config = crate::config::Config::parse_from(["ogis"]);
    crate::build_state(&config).expect("build_state failed in test")
}

#[tokio::test]
async fn c_route_renders_png() {
    let state = test_state();
    let reg = crate::wire::registry::Registry::load();
    let templates = crate::templates::load_templates();
    let p = crate::params::OgParams {
        title: Some("Hello".into()),
        description: None,
        subtitle: None,
        logo: None,
        image: None,
        template: None,
        signature: None,
        format: None,
        scale: None,
        quality: None,
        extra: std::collections::HashMap::new(),
    };
    let (blob, _) = crate::wire::encode(&p, reg, &templates, None).unwrap();

    let app = crate::routes::create_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/c/{blob}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let ct = res.headers().get("content-type").unwrap();
    assert_eq!(ct, "image/png");
}

#[tokio::test]
async fn c_route_rejects_garbage_with_400() {
    let state = test_state();
    let app = crate::routes::create_router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/c/not-a-real-blob")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
