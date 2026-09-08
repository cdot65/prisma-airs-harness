use codex_client::StreamResponse;
use codex_client::TransportError;
use codex_utils_home_dir::airs_session::AirsSessionGuard;
use codex_utils_home_dir::airs_session::current_airs_session_guard;
use futures::StreamExt;
use std::future::Future;
use std::time::Duration;

#[derive(Clone)]
pub(super) struct SessionGuard {
    pub(super) session: Result<Option<AirsSessionGuard>, String>,
}

impl SessionGuard {
    pub(super) fn current() -> Self {
        Self {
            session: current_airs_session_guard().map_err(|error| error.to_string()),
        }
    }

    pub(super) fn check(&self) -> Result<(), TransportError> {
        match &self.session {
            Ok(Some(guard)) => guard
                .check()
                .map_err(|error| TransportError::Build(error.to_string())),
            Ok(None) => Ok(()),
            Err(error) => Err(TransportError::Build(error.clone())),
        }
    }

    pub(super) async fn run<T>(
        &self,
        request: impl Future<Output = Result<T, TransportError>>,
    ) -> Result<T, TransportError> {
        self.check()?;
        if matches!(self.session, Ok(None)) {
            return request.await;
        }
        tokio::pin!(request);
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            tokio::select! {
                biased;
                _ = interval.tick() => self.check()?,
                result = &mut request => { self.check()?; return result; }
            }
        }
    }

    pub(super) fn stream(&self, mut response: StreamResponse) -> StreamResponse {
        if matches!(self.session, Ok(None)) {
            return response;
        }
        response.bytes = Box::pin(futures::stream::unfold(
            (response.bytes, self.clone(), false),
            |(mut bytes, guard, ended)| async move {
                if ended {
                    return None;
                }
                match guard.run(async { Ok(bytes.next().await) }).await {
                    Ok(Some(chunk)) => Some((chunk, (bytes, guard, false))),
                    Ok(None) => None,
                    Err(error) => Some((Err(error), (bytes, guard, true))),
                }
            },
        ));
        response
    }
}
