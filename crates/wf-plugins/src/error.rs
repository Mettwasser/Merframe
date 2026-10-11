use thiserror::Error;

pub type Result<T> = std::result::Result<T, PluginError>;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum PluginError {
    Lua(#[from] mlua::Error),
    Io(#[from] std::io::Error),
}
