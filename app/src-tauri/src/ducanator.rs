use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::Notify;
use wf_core::{ItemRecord, ducanator::DucanatorItems};

use crate::{
    commands::ducanator::DucanatorCancellationNotifier,
    error::{CommandError, CommandResult},
    state::AppState,
};

#[derive(Debug, Clone)]
pub struct ItemCounted {
    pub item: ItemRecord,
    pub count: u32,
}

fn get_mapped_items(
    state: &AppState,
    items: DucanatorItems,
) -> Result<Vec<ItemCounted>, CommandError> {
    let lock = state.core.lock()?;
    let item_table = lock.items();
    items
        .into_iter()
        .map(move |item| {
            Ok(ItemCounted {
                item: item_table
                    .get(&item.unique_name)
                    .cloned()
                    .ok_or_else(|| CommandError::from(""))?,
                count: item.count,
            })
        })
        .collect::<Result<Vec<ItemCounted>, CommandError>>()
}

pub async fn start_paste_listener(
    app: AppHandle,
    state: Arc<AppState>,
    items: DucanatorItems,
    cancel: DucanatorCancellationNotifier,
) -> CommandResult<()> {
    let items_mapped = get_mapped_items(&state, items)?;

    let mut idx = 0_usize;

    loop {
        tokio::select! {
            // TODO: Lua event for "DucatKiosk" item added to sell list - maybe pass down the entire list?
            () = cancel.0.notified() => break,
        };

        idx += 1;

        let Some(item) = items_mapped.get(idx) else {
            break;
        };

        app.clipboard().write_text(&item.item.name)?;
        app.emit("moveTo", idx)?;
    }

    Ok(())
}
