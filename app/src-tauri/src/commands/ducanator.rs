use std::sync::Arc;

use tauri::{AppHandle, State};
use wf_core::ducanator::DucanatorItems;

use crate::{
    commands::{Shared, ready},
    ducanator,
    error::CommandResult,
};

#[derive(Debug, Clone)]
pub struct DucanatorCancellationNotifier(pub Arc<tokio::sync::Notify>);

#[tauri::command]
pub async fn start_sell_run(
    app: AppHandle,
    state: Shared<'_>,
    items: DucanatorItems,
    cancel: State<'_, DucanatorCancellationNotifier>,
) -> CommandResult<()> {
    let app_state = ready(&state).await?;

    tokio::spawn(ducanator::start_paste_listener(
        app,
        app_state,
        items,
        cancel.inner().clone(),
    ));

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
pub fn stop_sell_run(cancel: State<'_, DucanatorCancellationNotifier>) -> CommandResult<()> {
    cancel.inner().0.notify_one();
    Ok(())
}
