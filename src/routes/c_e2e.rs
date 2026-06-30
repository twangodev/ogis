use axum::body::Body;
use axum::http::{Request, StatusCode};
use clap::Parser;
use tower::ServiceExt; // oneshot

fn test_state() -> crate::AppState {
    let config = crate::config::Config::parse_from(["ogis"]);
    crate::build_state(&config).expect("build_state failed in test")
}

fn test_state_with_hmac(secret: &str) -> crate::AppState {
    let config = crate::config::Config::parse_from(["ogis", "--secret", secret]);
    crate::build_state(&config).expect("build_state failed in test")
}

#[tokio::test]
async fn c_route_renders_png() {
    let state = test_state();
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
    let (blob, _) = crate::wire::encode(&p, None).unwrap();

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

#[tokio::test]
async fn c_route_signed_hmac() {
    let secret = b"testsecret";
    let state = test_state_with_hmac("testsecret");
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
    let (blob, sig) = crate::wire::encode(&p, Some(secret)).unwrap();
    let sig = sig.unwrap();

    // Signed request → 200 + image/png
    let app = crate::routes::create_router(state.clone());
    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/c/{blob}/{sig}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers().get("content-type").unwrap(), "image/png");

    // Missing sig → 401
    let app2 = crate::routes::create_router(state);
    let res2 = app2
        .oneshot(
            Request::builder()
                .uri(format!("/c/{blob}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn c_route_admits_boundary_values() {
    // A request at validation boundaries (title length == max_input_length, quality
    // at its upper bound) must render 200 through the full route, so a future
    // tightening of a bound that would 400 an already-published URL fails CI
    // (spec §9 "boundary blob → 200" vectors). Note: decode() alone skips validate,
    // so this MUST go through the route.
    let state = test_state();
    let max = state.max_input_length;
    let p = crate::params::OgParams {
        title: Some("x".repeat(max)),
        description: None,
        subtitle: None,
        logo: None,
        image: None,
        template: None,
        signature: None,
        format: None,
        scale: None,
        quality: Some(100), // upper bound of 1..=100
        extra: std::collections::HashMap::new(),
    };
    let (blob, _) = crate::wire::encode(&p, None).unwrap();
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
}
