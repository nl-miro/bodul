pub mod io {
    pub use super::security::BearerAuth;
}

mod security {
    use poem::{Endpoint, Middleware, Request, Result, http::StatusCode};
    use subtle::ConstantTimeEq;

    pub struct BearerAuth {
        token: String,
    }

    impl BearerAuth {
        pub fn new(token: String) -> Self {
            Self { token }
        }
    }

    impl<E: Endpoint> Middleware<E> for BearerAuth {
        type Output = BearerAuthEndpoint<E>;

        fn transform(&self, ep: E) -> Self::Output {
            BearerAuthEndpoint {
                inner: ep,
                token: self.token.clone(),
            }
        }
    }

    pub struct BearerAuthEndpoint<E> {
        inner: E,
        token: String,
    }

    impl<E: Endpoint> Endpoint for BearerAuthEndpoint<E> {
        type Output = E::Output;

        async fn call(&self, req: Request) -> Result<Self::Output> {
            let provided = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .unwrap_or("");

            if bool::from(provided.as_bytes().ct_eq(self.token.as_bytes())) {
                self.inner.call(req).await
            } else {
                Err(poem::Error::from_status(StatusCode::UNAUTHORIZED))
            }
        }
    }
}
