//! Apply the AIRS environment logout boundary below MCP headers and redirects.
use codex_exec_server::ExecServerError;
use codex_exec_server::HttpClient;
use codex_exec_server::HttpRequestParams;
use codex_exec_server::HttpRequestResponse;
use codex_exec_server::HttpResponseBodyStream;
use codex_utils_home_dir::airs_session::AirsSessionGuard;
use codex_utils_home_dir::airs_session::current_airs_session_guard;
use futures::FutureExt;
use futures::future::BoxFuture;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

struct AirsHttpClient {
    inner: Arc<dyn HttpClient>,
    session: Result<Option<AirsSessionGuard>, String>,
}

pub(crate) fn guard_http_client(inner: Arc<dyn HttpClient>) -> Arc<dyn HttpClient> {
    let session = current_airs_session_guard().map_err(|error| error.to_string());
    if matches!(session, Ok(None)) {
        return inner;
    }
    Arc::new(AirsHttpClient { inner, session })
}

impl AirsHttpClient {
    async fn request<T>(
        &self,
        future: impl Future<Output = Result<T, ExecServerError>>,
    ) -> Result<T, ExecServerError> {
        let guard = match &self.session {
            Ok(Some(guard)) => guard,
            Ok(None) => return future.await,
            Err(error) => return Err(ExecServerError::HttpRequest(error.clone())),
        };
        let check = || {
            guard
                .check()
                .map_err(|error| ExecServerError::HttpRequest(error.to_string()))
        };
        check()?;
        tokio::pin!(future);
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            tokio::select! {
                biased;
                _ = interval.tick() => check()?,
                result = &mut future => { check()?; return result; }
            }
        }
    }
}

impl HttpClient for AirsHttpClient {
    fn http_request(
        &self,
        params: HttpRequestParams,
    ) -> BoxFuture<'_, Result<HttpRequestResponse, ExecServerError>> {
        self.request(self.inner.http_request(params)).boxed()
    }

    fn http_request_stream(
        &self,
        params: HttpRequestParams,
    ) -> BoxFuture<'_, Result<(HttpRequestResponse, HttpResponseBodyStream), ExecServerError>> {
        self.request(self.inner.http_request_stream(params)).boxed()
    }
}

#[cfg(test)]
#[path = "airs_http_tests.rs"]
mod tests;
