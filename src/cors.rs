use crate::config::CorsConfig;
use axum::http::{
    Method,
    header::{AUTHORIZATION, CONTENT_TYPE, LOCATION, WWW_AUTHENTICATE},
};
use std::time::Duration;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn layer(config: &CorsConfig) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(config.allowed_origins.clone()))
        .allow_methods([
            Method::GET,
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .expose_headers([LOCATION, WWW_AUTHENTICATE])
        .max_age(Duration::from_secs(600))
}

#[cfg(test)]
mod tests {
    use crate::{
        app_state::AppState,
        config::{AuthConfig, CorsConfig},
    };
    use axum::{
        Router,
        body::Body,
        http::{
            HeaderMap, HeaderName, Method, Request, StatusCode,
            header::{
                ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
                ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
                ACCESS_CONTROL_EXPOSE_HEADERS, ACCESS_CONTROL_MAX_AGE,
                ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, VARY,
                WWW_AUTHENTICATE,
            },
        },
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    async fn test_app(config: &CorsConfig) -> Router {
        // These requests use preflight or fail before any database lookup.
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/cors_test_unused")
            .unwrap();
        let state = AppState::new(
            pool,
            &AuthConfig {
                jwt_secret: "cors-test-only-secret-with-at-least-32-bytes".into(),
                jwt_ttl_seconds: 3600,
            },
        )
        .await
        .unwrap();
        crate::app(state, config)
    }

    fn contains(headers: &HeaderMap, name: HeaderName, value: &str) -> bool {
        headers
            .get(name)
            .unwrap()
            .to_str()
            .unwrap()
            .split(',')
            .any(|field| field.trim().eq_ignore_ascii_case(value))
    }

    fn preflight(origin: &str, path: &str) -> Request<Body> {
        Request::builder()
            .method(Method::OPTIONS)
            .uri(path)
            .header(ORIGIN, origin)
            .header(ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(ACCESS_CONTROL_REQUEST_HEADERS, "authorization,content-type")
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn preflight_allows_configured_origins_without_authentication() {
        let config = CorsConfig::from_origins(
            "http://localhost:5173,http://127.0.0.1:5173,https://portfolio.example",
        )
        .unwrap();
        let app = test_app(&config).await;
        for origin in [
            "http://localhost:5173",
            "http://127.0.0.1:5173",
            "https://portfolio.example",
        ] {
            for path in ["/hotels", "/auth/login", "/auth/register"] {
                let response = app.clone().oneshot(preflight(origin, path)).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(response.headers()[ACCESS_CONTROL_ALLOW_ORIGIN], origin);
                assert!(contains(
                    response.headers(),
                    ACCESS_CONTROL_ALLOW_HEADERS,
                    "authorization"
                ));
                assert!(contains(
                    response.headers(),
                    ACCESS_CONTROL_ALLOW_HEADERS,
                    "content-type"
                ));
                for method in ["GET", "HEAD", "POST", "PUT", "DELETE", "OPTIONS"] {
                    assert!(contains(
                        response.headers(),
                        ACCESS_CONTROL_ALLOW_METHODS,
                        method
                    ));
                }
                assert!(!contains(
                    response.headers(),
                    ACCESS_CONTROL_ALLOW_METHODS,
                    "TRACE"
                ));
                assert!(!contains(
                    response.headers(),
                    ACCESS_CONTROL_ALLOW_HEADERS,
                    "x-arbitrary-header"
                ));
                assert!(
                    !response
                        .headers()
                        .contains_key(ACCESS_CONTROL_ALLOW_CREDENTIALS)
                );
                assert_eq!(response.headers()[ACCESS_CONTROL_MAX_AGE], "600");
                assert!(contains(response.headers(), VARY, "origin"));
            }
        }
    }

    #[tokio::test]
    async fn unlisted_origins_do_not_receive_browser_access() {
        let app = test_app(&CorsConfig::default()).await;
        for origin in [
            "https://untrusted.example",
            "http://localhost:5174",
            "http://localhost:5173.evil.example",
            "null",
        ] {
            let response = app
                .clone()
                .oneshot(preflight(origin, "/hotels"))
                .await
                .unwrap();
            assert!(!response.headers().contains_key(ACCESS_CONTROL_ALLOW_ORIGIN));
        }
        let app = test_app(&CorsConfig::from_origins("").unwrap()).await;
        let response = app
            .oneshot(preflight("http://localhost:5173", "/hotels"))
            .await
            .unwrap();
        assert!(!response.headers().contains_key(ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[tokio::test]
    async fn authentication_errors_include_cors_headers() {
        let app = test_app(&CorsConfig::default()).await;
        let request = Request::builder()
            .uri("/auth/me")
            .header(ORIGIN, "http://localhost:5173")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()[WWW_AUTHENTICATE], "Bearer");
        assert_eq!(
            response.headers()[ACCESS_CONTROL_ALLOW_ORIGIN],
            "http://localhost:5173"
        );
        assert!(contains(
            response.headers(),
            ACCESS_CONTROL_EXPOSE_HEADERS,
            "location"
        ));
        assert!(contains(
            response.headers(),
            ACCESS_CONTROL_EXPOSE_HEADERS,
            "www-authenticate"
        ));
        let request = Request::builder()
            .uri("/auth/me")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(!response.headers().contains_key(ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[test]
    fn origins_are_validated_normalized_and_deduplicated() {
        let config = CorsConfig::from_origins(
            " http://localhost:5173/ ,http://localhost:5173,https://EXAMPLE.com:443 ",
        )
        .unwrap();
        assert_eq!(config.allowed_origins.len(), 2);
        assert_eq!(config.allowed_origins[0], "http://localhost:5173");
        assert_eq!(config.allowed_origins[1], "https://example.com");
        assert_eq!(CorsConfig::default().allowed_origins.len(), 2);
        assert!(
            CorsConfig::from_origins(" ")
                .unwrap()
                .allowed_origins
                .is_empty()
        );
    }

    #[test]
    fn invalid_origins_are_rejected() {
        for origin in [
            "*",
            "null",
            "localhost:5173",
            "file:///tmp/frontend",
            "ftp://example.com",
            "https://user:password@example.com",
            "https://example.com/path",
            "https://example.com?query=1",
            "https://example.com#fragment",
            "http://localhost:5173,",
            "https://",
            "http://local\nhost:5173",
            "http://localhost:5173\r\nInjected: value",
        ] {
            assert!(
                CorsConfig::from_origins(origin).is_err(),
                "accepted invalid origin"
            );
        }
    }
}
