//! Own the desktop-only command guard after CLI environment setup and before
//! stdin bootstrap. Losing either supervision path retires the runtime.
use std::future::Future;
use std::time::Duration;

pub(super) async fn run(runtime: impl Future<Output = anyhow::Result<()>>) -> anyhow::Result<()> {
    let supervisor = codex_utils_pty::start_desktop_supervisor(&std::env::current_exe()?)?;
    let health = async {
        loop {
            tokio::time::sleep(Duration::from_millis(25)).await;
            if supervisor.check_health().is_err() {
                return Err::<(), anyhow::Error>(anyhow::anyhow!(
                    "Desktop command supervision unavailable"
                ));
            }
        }
    };
    tokio::select! {
        result = runtime => result,
        result = health => result,
    }
}
