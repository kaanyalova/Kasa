use std::mem;

use anyhow::anyhow;
use futures::{FutureExt, TryStreamExt};
use gpui::http_client::{self, HttpClient, RedirectPolicy, http};
use reqwest::redirect;
use tokio_util::{
    compat::FuturesAsyncReadCompatExt,
    io::{ReaderStream, StreamReader},
};

pub struct ReqwestClient {
    pub client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

impl HttpClient for ReqwestClient {
    fn type_name(&self) -> &'static str {
        todo!()
    }

    fn user_agent(&self) -> Option<&reqwest::header::HeaderValue> {
        todo!()
    }

    fn send(
        &self,
        req: gpui::http_client::http::Request<gpui::http_client::AsyncBody>,
    ) -> futures::future::BoxFuture<
        'static,
        anyhow::Result<gpui::http_client::Response<gpui::http_client::AsyncBody>>,
    > {
        let (parts, body) = req.into_parts();

        let mut request = self.client.request(parts.method, parts.uri.to_string());
        request = request.headers(parts.headers);

        if let Some(redirect_policy) = parts.extensions.get::<RedirectPolicy>() {
            //request = request.(match redirect_policy {
            //    RedirectPolicy::NoFollow => redirect::Policy::none(),
            //    RedirectPolicy::FollowLimit(limit) => redirect::Policy::limited(*limit as usize),
            //    RedirectPolicy::FollowAll => redirect::Policy::limited(100),
            //});
        }
        let request = request.body(match body.0 {
            http_client::Inner::Empty => reqwest::Body::default(),
            http_client::Inner::Bytes(cursor) => cursor.into_inner().into(),
            http_client::Inner::AsyncReader(stream) => {
                reqwest::Body::wrap_stream(ReaderStream::new(stream.compat()))
            }
        });

        async move {
            let mut response = request.send().await?;

            let headers = mem::take(response.headers_mut());
            let mut builder = http::Response::builder()
                .status(response.status().as_u16())
                .version(response.version());
            *builder.headers_mut().unwrap() = headers;

            let bytes = response
                .bytes_stream()
                .map_err(tokio::io::Error::other)
                .into_async_read();
            let body = http_client::AsyncBody::from_reader(bytes);

            builder.body(body).map_err(|e| anyhow!(e))
        }
        .boxed()
    }

    fn proxy(&self) -> Option<&reqwest::Url> {
        None
    }
}
