//! The usher command line (Story 1.5): the `inventory` verb. A driving adapter over
//! the ports in `usher-core`; `crates/usher` hands `usher <command>` to it and holds no
//! logic of its own.

#![forbid(unsafe_code)]

mod inventory;
mod time;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

use usher_core::{SystemRoot, UsherError};

pub use inventory::{inventory, inventory_with};

/// The launch variables the verb asked for, as a name-to-value lookup. In production
/// [`LaunchEnvironment::from_process`] fills it with the variables knowledge names and
/// nothing else, so no other part of the process environment is held.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchEnvironment {
    values: BTreeMap<String, String>,
}

impl LaunchEnvironment {
    /// A lookup holding exactly these pairs, for tests.
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        LaunchEnvironment {
            values: pairs
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
        }
    }

    /// Reads the named variables from the process environment. A variable that is unset
    /// is absent; one that is set to text that is not valid UTF-8 is refused, naming the
    /// variable and never its value.
    pub fn from_process(names: &[&str]) -> Result<Self, UsherError> {
        Self::from_lookup(names, |name| std::env::var_os(name))
    }

    /// As [`LaunchEnvironment::from_process`], over any lookup, so a test can hand in
    /// values a process environment cannot be made to hold. The lookup is asked for
    /// each of `names` and nothing else.
    pub fn from_lookup(
        names: &[&str],
        lookup: impl Fn(&str) -> Option<OsString>,
    ) -> Result<Self, UsherError> {
        let mut values = BTreeMap::new();
        for name in names {
            let Some(raw) = lookup(name) else { continue };
            let Ok(text) = raw.into_string() else {
                return Err(UsherError::new(
                    "read the launch environment",
                    format!("the value of {name} is valid UTF-8 text"),
                    format!("{name} is set to text that is not valid UTF-8"),
                    format!(
                        "set {name} to valid UTF-8 text or unset it, then run `usher inventory` again"
                    ),
                ));
            };
            values.insert((*name).to_string(), text);
        }
        Ok(LaunchEnvironment { values })
    }

    /// The value of `name`, when it was kept.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }
}

/// Everything `usher inventory` reads from the process, named so tests can pass their
/// own. In production [`InventoryEnv::from_process`] fills it.
#[derive(Debug, Clone)]
pub struct InventoryEnv {
    /// The host id.
    pub host: String,
    /// The OS user's name.
    pub user: String,
    /// The OS user's account home, for `${HOME}`.
    pub home: PathBuf,
    /// The directory the command runs in: the launch directory.
    pub launch_dir: PathBuf,
    /// The launch environment's variables that harness knowledge names.
    pub launch: LaunchEnvironment,
    /// The launch environment's `PATH`.
    pub path: Option<OsString>,
    /// The state directory the store lives in.
    pub state_dir: PathBuf,
    /// Now, as `YYYY-MM-DDTHH:MM:SSZ`, for the run record.
    pub now: String,
    /// The platform whose sources are read: the build's own in production.
    pub platform: usher_knowledge::Platform,
}

impl InventoryEnv {
    /// Reads the environment of this process: the host, the account the process runs as,
    /// the launch directory, the launch variables knowledge names, `PATH` and the clock.
    pub fn from_process() -> Result<Self, UsherError> {
        inventory::env_from_process()
    }
}

/// The system root the `inventory` verb builds its adapter with: the host's own.
pub fn inventory_system_root() -> SystemRoot {
    SystemRoot::host()
}

/// The platform the `inventory` verb reads sources for: the build's own, with WSL as
/// Linux.
pub fn inventory_platform() -> usher_knowledge::Platform {
    if cfg!(target_os = "macos") {
        usher_knowledge::Platform::Macos
    } else if cfg!(target_os = "windows") {
        usher_knowledge::Platform::Windows
    } else {
        usher_knowledge::Platform::Linux
    }
}

const NO_ARGUMENTS: &str = "usher inventory takes no arguments";
const INVENTORY_NEXT: &str = "run `usher inventory`";

/// Checks the arguments of a command before anything is read, and says which verb they
/// name.
fn parse_command(args: &[String]) -> Result<(), UsherError> {
    match args.first().map(String::as_str) {
        Some("inventory") if args.len() == 1 => Ok(()),
        Some("inventory") => Err(UsherError::new(
            "run inventory",
            "the verb takes no arguments",
            format!(
                "{NO_ARGUMENTS}; got `{}`",
                args[1..].join(" ").escape_debug()
            ),
            INVENTORY_NEXT,
        )),
        Some(other) => Err(UsherError::new(
            "select command",
            "the command is one this build has",
            format!("`{}` is not a usher command", other.escape_debug()),
            "run `usher inventory` or `usher --version`",
        )),
        None => Err(UsherError::new(
            "select command",
            "a command was given",
            "no command",
            "run `usher inventory` or `usher --version`",
        )),
    }
}

/// Hands one command and its arguments to its verb.
pub fn run(args: &[String], env: &InventoryEnv) -> Result<String, UsherError> {
    parse_command(args)?;
    inventory(env, inventory_system_root())
}

/// As [`run`], reading the environment from this process after the arguments check out,
/// so a refused argument list reads nothing.
pub fn run_process(args: &[String]) -> Result<String, UsherError> {
    parse_command(args)?;
    run(args, &InventoryEnv::from_process()?)
}
