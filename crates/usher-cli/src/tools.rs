//! The `tools` verb (story 2.1): the installed tools, their source and version, read from
//! the installers' own files.

use usher_core::{InstallerPaths, ToolRow, ToolSpec, UsherError};

/// What the verb reads: the tools to look for and where the installers keep their state.
#[derive(Debug, Clone)]
pub struct ToolsContext {
    /// The tools to look for.
    pub specs: Vec<ToolSpec>,
    /// Where the installers keep their state.
    pub paths: InstallerPaths,
}

/// Runs the verb over `context`.
pub fn run_tools(args: &[String], context: &ToolsContext) -> Result<String, UsherError> {
    let _ = (args, context);
    Ok(String::new())
}

/// Reads one brew tool from the Cellar, the link directory, the receipts and the taps.
pub fn read_brew(spec: &ToolSpec, paths: &InstallerPaths) -> Vec<ToolRow> {
    let _ = (spec, paths);
    Vec::new()
}

/// Reads one mise tool from the `installs` directory.
pub fn read_mise(spec: &ToolSpec, paths: &InstallerPaths) -> Vec<ToolRow> {
    let _ = (spec, paths);
    Vec::new()
}

/// Reads every tool from the installer that owns it, one row per installed entry of a
/// name, or one row for a name with none installed.
pub fn read_tools(specs: &[ToolSpec], paths: &InstallerPaths) -> Vec<ToolRow> {
    let _ = (specs, paths);
    Vec::new()
}
