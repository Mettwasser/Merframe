use std::{path::Path, sync::Arc};

use mlua::Lua;
use tokio::sync::mpsc;
use wf_core::CoreEvent;
use wf_plugins::Plugin;

#[derive(Debug, Clone)]
pub struct PluginManager {
    pub lua: Arc<Lua>,
    pub loaded_plugins: Arc<[Plugin]>,
    pub dispatcher_tx: mpsc::UnboundedSender<CoreEvent>,
}

impl PluginManager {
    pub fn new_with_spawned_dispatcher(plugins_dir: &Path) -> wf_plugins::error::Result<Self> {
        let lua = Arc::new(Lua::new());
        let plugins = wf_plugins::load(&lua, plugins_dir)?;

        let (tx, rx) = mpsc::unbounded_channel();

        tauri::async_runtime::spawn(wf_plugins::plugin_event_dispatcher(Arc::clone(&lua), rx));

        Ok(Self {
            lua,
            loaded_plugins: plugins.into(),
            dispatcher_tx: tx,
        })
    }
}
