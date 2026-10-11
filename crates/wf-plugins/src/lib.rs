pub mod error;

use crate::error::Result;
use mlua::{Function, Lua, LuaSerdeExt, Table};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use strum::IntoDiscriminant;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::info;
use wf_core::CoreEvent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
}

pub fn load(lua: &Lua, plugins_dir: &Path) -> Result<Vec<Plugin>> {
    setup_plugin_api(lua)?;
    load_all_plugins(lua, plugins_dir)
}

pub async fn plugin_event_dispatcher(lua: Arc<Lua>, mut rx: UnboundedReceiver<CoreEvent>) {
    while let Some(evt) = rx.recv().await {
        let lua = lua.clone();
        let event_to_dispatch = &evt.discriminant().to_string();

        info!(%event_to_dispatch, "dispatching event");
        let result = dispatch_event(&lua, event_to_dispatch, &evt);

        if let Err(err) = result {
            tracing::error!("Failed to execute plugin listener: {err}");
        }
    }
}

fn dispatch_event<T: Serialize + Send + 'static>(
    lua: &Lua,
    event_name: &str,
    payload: &T,
) -> std::result::Result<(), mlua::Error> {
    let event_name = event_name.to_string();

    let Ok(listeners) = lua.named_registry_value::<Table>("event_listeners") else {
        return Ok(());
    };

    let Ok(list) = listeners.get::<Table>(event_name.clone()) else {
        return Ok(());
    };

    let lua_payload = lua.to_value(&payload)?;

    for pair in list.sequence_values::<Function>() {
        let Ok(callback) = pair else {
            continue;
        };

        let payload_clone = lua_payload.clone();

        tokio::spawn(async move {
            if let Err(e) = callback.call_async::<()>(payload_clone).await {
                tracing::error!("Error in async Lua plugin handler: {}", e);
            }
        });
    }

    Ok(())
}

fn setup_plugin_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    let app_table = lua.create_table()?;

    // Internal table inside the Lua registry to hold our event listeners safely
    let listeners_registry = lua.create_table()?;
    lua.set_named_registry_value("event_listeners", listeners_registry)?;

    // App.on("event_name", function(data) ... end)
    app_table.set(
        "on",
        lua.create_function(|lua, (event_name, callback): (String, Function)| {
            let listeners: Table = lua.named_registry_value("event_listeners")?;

            // Get or create the list of callbacks for this event name
            let list = if let Ok(l) = listeners.get::<Table>(event_name.clone()) {
                l
            } else {
                let new_list = lua.create_table()?;
                listeners.set(event_name.clone(), new_list.clone())?;
                new_list
            };

            list.push(callback)?;

            Ok(())
        })?,
    )?;

    globals.set("App", app_table)?;
    Ok(())
}

fn load_all_plugins(lua: &Lua, plugins_dir: &Path) -> Result<Vec<Plugin>> {
    let mut plugins = Vec::new();

    if !plugins_dir.exists() {
        return Ok(plugins);
    }

    for entry in std::fs::read_dir(plugins_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let main_path = path.join("main.lua");
            if main_path.is_file()
                && let Some(dir_name) = path.file_name()
            {
                let script_content = std::fs::read_to_string(&main_path)?;
                let plugin_name = dir_name.to_string_lossy();

                // Create a unique environment table for this specific plugin
                let plugin_env = lua.create_table()?;

                // Give the plugin access to global functions (print, math, App, etc.) - set in `setup_plugin_api`
                let globals = lua.globals();
                plugin_env.set_metatable(Some(
                    lua.create_table_from([("__index", mlua::Value::Table(globals))])?,
                ))?;

                // Load the script chunk and bind it to the plugin's private environment
                let chunk = lua
                    .load(&script_content)
                    .set_name(plugin_name.to_string())
                    .set_environment(plugin_env);

                // Execute the plugin file - registers listeners
                if let Err(e) = chunk.exec() {
                    tracing::error!("Failed to load plugin {}: {}", plugin_name, e);
                } else {
                    plugins.push(Plugin {
                        name: dir_name.to_string_lossy().to_string(),
                    });
                    tracing::info!("Loaded plugin: {}", plugin_name);
                }
            }
        }
    }

    Ok(plugins)
}
