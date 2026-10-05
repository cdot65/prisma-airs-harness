//! Bounded, typed desktop replay commands carried exclusively by the in-process transport.

use super::DesktopReplay;
use super::DesktopRestoreParams;
use super::IN_PROCESS_CONNECTION_ID;
use super::InProcessClientHandle;
use super::InProcessClientMessage;
use super::InProcessClientSender;
use crate::message_processor::ConnectionSessionState;
use crate::message_processor::MessageProcessor;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::ThreadResumeResponse;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::io::Result as IoResult;
use tokio::sync::oneshot;

type ReplayResult<T> = Result<T, JSONRPCErrorError>;

pub(super) enum DesktopHistoryCommand {
    Export {
        thread_id: String,
        response: oneshot::Sender<ReplayResult<DesktopReplay>>,
    },
    Restore {
        params: Box<DesktopRestoreParams>,
        response: oneshot::Sender<ReplayResult<ThreadResumeResponse>>,
    },
}

impl DesktopHistoryCommand {
    pub(super) fn fail(self, error: JSONRPCErrorError) {
        match self {
            Self::Export { response, .. } => {
                let _ = response.send(Err(error));
            }
            Self::Restore { response, .. } => {
                let _ = response.send(Err(error));
            }
        }
    }

    pub(super) async fn execute(
        self,
        processor: &MessageProcessor,
        session: &ConnectionSessionState,
    ) {
        match self {
            Self::Export {
                thread_id,
                response,
            } => {
                let _ = response.send(processor.export_desktop_history(thread_id).await);
            }
            Self::Restore { params, response } => {
                let _ = response.send(
                    processor
                        .restore_desktop_history(*params, IN_PROCESS_CONNECTION_ID, session)
                        .await,
                );
            }
        }
    }
}

impl InProcessClientSender {
    /// Exports an idle thread's full RAM history for host-owned encryption.
    pub async fn export_desktop_history(
        &self,
        thread_id: String,
    ) -> IoResult<ReplayResult<DesktopReplay>> {
        let (response, receiver) = oneshot::channel();
        self.try_send_client_message(InProcessClientMessage::DesktopHistory(
            DesktopHistoryCommand::Export {
                thread_id,
                response,
            },
        ))?;
        receiver.await.map_err(|_| {
            IoError::new(
                ErrorKind::BrokenPipe,
                "desktop history export channel closed",
            )
        })
    }

    /// Restores encrypted-store plaintext into RAM without starting a turn or executing old calls.
    ///
    /// The host must authenticate the encrypted record and bind it to its current capability bootstrap.
    pub async fn restore_desktop_history(
        &self,
        params: DesktopRestoreParams,
    ) -> IoResult<ReplayResult<ThreadResumeResponse>> {
        let (response, receiver) = oneshot::channel();
        self.try_send_client_message(InProcessClientMessage::DesktopHistory(
            DesktopHistoryCommand::Restore {
                params: Box::new(params),
                response,
            },
        ))?;
        receiver.await.map_err(|_| {
            IoError::new(
                ErrorKind::BrokenPipe,
                "desktop history restore channel closed",
            )
        })
    }
}

impl InProcessClientHandle {
    /// Exports an idle thread's sensitive RAM history to the owned host.
    pub async fn export_desktop_history(
        &self,
        thread_id: String,
    ) -> IoResult<ReplayResult<DesktopReplay>> {
        self.client.export_desktop_history(thread_id).await
    }

    /// Passively restores a protected history and subscribes this client to its live events.
    pub async fn restore_desktop_history(
        &self,
        params: DesktopRestoreParams,
    ) -> IoResult<ReplayResult<ThreadResumeResponse>> {
        self.client.restore_desktop_history(params).await
    }
}

#[cfg(test)]
#[path = "in_process_desktop_history_tests.rs"]
mod tests;
