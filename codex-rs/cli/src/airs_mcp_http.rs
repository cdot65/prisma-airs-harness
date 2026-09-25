//! Observe only HTTP outcomes for private MCP recovery; delegate all request bytes unchanged.
use codex_exec_server::ExecServerError;
use codex_exec_server::HttpClient;
use codex_exec_server::HttpRequestParams;
use codex_exec_server::HttpRequestResponse;
use codex_exec_server::HttpResponseBodyStream;
use codex_protocol::airs_mcp_failure::FailureCode;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::sync::Mutex;

pub(crate) struct McpHttpDiagnostics {
    inner: Arc<dyn HttpClient>,
    failure: Mutex<Option<FailureCode>>,
}

impl McpHttpDiagnostics {
    pub(crate) fn new(inner: Arc<dyn HttpClient>) -> Self {
        Self {
            inner,
            failure: Mutex::new(None),
        }
    }

    pub(crate) fn failure(&self) -> Option<FailureCode> {
        *self
            .failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn observe(&self, status: Result<u16, &ExecServerError>) {
        let failure = match status {
            Ok(403) => Some(FailureCode::PermissionDenied),
            Ok(446) => Some(FailureCode::PolicyDenied),
            Ok(429) => Some(FailureCode::RateLimited),
            Ok(408 | 504) => Some(FailureCode::TimedOut),
            Ok(500..=599) => Some(FailureCode::RemoteUnavailable),
            Ok(400..=499) => Some(FailureCode::Authorization),
            Ok(_) => None,
            Err(_) => Some(FailureCode::Transport),
        };
        *self
            .failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = failure;
    }
}

impl HttpClient for McpHttpDiagnostics {
    fn http_request(
        &self,
        params: HttpRequestParams,
    ) -> BoxFuture<'_, Result<HttpRequestResponse, ExecServerError>> {
        Box::pin(async move {
            let result = self.inner.http_request(params).await;
            self.observe(result.as_ref().map(|response| response.status));
            result
        })
    }

    fn http_request_stream(
        &self,
        params: HttpRequestParams,
    ) -> BoxFuture<'_, Result<(HttpRequestResponse, HttpResponseBodyStream), ExecServerError>> {
        Box::pin(async move {
            let result = self.inner.http_request_stream(params).await;
            self.observe(result.as_ref().map(|(response, _)| response.status));
            result
        })
    }
}

#[cfg(test)]
#[path = "airs_mcp_http_tests.rs"]
mod tests;
