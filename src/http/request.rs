use std::{convert::Infallible, future::Future};

use http::{request::Parts, HeaderMap, HeaderValue};

use crate::http::body::Body;

use super::response::into_response::IntoResponse;

pub struct Request<T = Body>(pub http::Request<T>);

impl<T> Request<T> {
    #[inline]
    pub fn new(body: T) -> Self {
        Self(http::Request::new(body))
    }

    #[inline]
    pub fn from_parts(parts: Parts, body: T) -> Self {
        Self(http::Request::from_parts(parts, body))
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        self.0.headers()
    }

    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        self.0.headers_mut()
    }

    #[inline]
    pub fn body(&self) -> &T {
        self.0.body()
    }

    #[inline]
    pub fn into_body(self) -> T {
        self.0.into_body()
    }

    #[inline]
    pub fn into_parts(self) -> (Parts, T) {
        self.0.into_parts()
    }

    #[inline]
    pub fn map<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        Request(self.0.map(f))
    }
}

// TODO: Move this into the extract module.

// FROM AXUM. Ensures that we can have seperate and nicely partitioned
// implementations of the from_request trait for different types.
mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaParts {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaRequest {}
}

pub trait FromRequestParts<S>: Sized {
    type Error: IntoResponse;
    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

pub trait FromRequest<S, M = private::ViaRequest>: Sized {
    type Error: IntoResponse;

    fn from_request(
        req: Request,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

impl<S, T> FromRequest<S, private::ViaParts> for T
where
    S: Send + Sync,
    T: FromRequestParts<S>,
{
    type Error = <Self as FromRequestParts<S>>::Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Error> {
        let (mut parts, _) = req.0.into_parts();
        Self::from_request_parts(&mut parts, state).await
    }
}

impl<S, T> FromRequestParts<S> for Option<T>
where
    T: FromRequestParts<S>,
    S: Send + Sync,
{
    type Error = Infallible;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Option<T>, Self::Error> {
        Ok(T::from_request_parts(parts, state).await.ok())
    }
}

impl<S, T> FromRequest<S> for Option<T>
where
    T: FromRequest<S>,
    S: Send + Sync,
{
    type Error = Infallible;
    async fn from_request(req: Request, state: &S) -> Result<Option<T>, Self::Error> {
        Ok(T::from_request(req, state).await.ok())
    }
}

impl<S, T> FromRequestParts<S> for Result<T, T::Error>
where
    T: FromRequestParts<S>,
    S: Send + Sync,
{
    type Error = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Error> {
        Ok(T::from_request_parts(parts, state).await)
    }
}

impl<S, T> FromRequest<S> for Result<T, T::Error>
where
    T: FromRequest<S>,
    S: Send + Sync,
{
    type Error = Infallible;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Error> {
        Ok(T::from_request(req, state).await)
    }
}

impl<S> FromRequest<S> for Request
where
    S: Send + Sync,
{
    type Error = Infallible;

    async fn from_request(req: Request, _: &S) -> Result<Self, Self::Error> {
        Ok(req)
    }
}
