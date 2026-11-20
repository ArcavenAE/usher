//! The usher command line (Story 1.5): the `inventory` verb. A driving adapter over
//! the ports in `usher-core`; `crates/usher` hands `usher <command>` to it and holds no
//! logic of its own.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::path::PathBuf;

use usher_core::{SystemRoot, UsherError};

/// The launch variables the verb asked for, as a name-to-value lookup. In production
/// [`LaunchEnvironment::from_process`] fills it with the variables knowledge names and
/// nothing else, so no other part of the process environment is held.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchEnvironment {
    values: std::collections::BTreeMap<String, String>,
}

impl LaunchEnvironment {
    /// A lookup holding exactly these pairs, for tests.
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let _ = pairs;
        LaunchEnvironment::default()
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
        let _ = (names, lookup);
        Ok(LaunchEnvironment::default())
    }

    /// The value of `name`, when it was kept.
    pub fn get(&self, name: &str) -> Option<&str> {
        let _ = name;
        None
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

/// The system root the `inventory` verb builds its adapter with: the host's own.
pub fn inventory_system_root() -> SystemRoot {
    SystemRoot::at(PathBuf::from("/scratch-stub"))
}

/// Runs `usher inventory` against `env`, placing absolute knowledge paths under `root`,
/// and returns the text to print. Only tests pass a root other than the host's.
pub fn inventory(env: &InventoryEnv, root: SystemRoot) -> Result<String, UsherError> {
    let _ = (env, root);
    Ok(String::new())
}

/// The platform the `inventory` verb reads sources for: the build's own, with WSL as
/// Linux.
pub fn inventory_platform() -> usher_knowledge::Platform {
    usher_knowledge::Platform::Windows
}

/// As [`inventory`], over a given knowledge, for tests that need a fact the seed does not
/// hold. A [`usher_knowledge::Knowledge`] is the embedded file or a signed update, so no
/// unsigned knowledge reaches the verb.
pub fn inventory_with(
    env: &InventoryEnv,
    root: SystemRoot,
    knowledge: &usher_knowledge::Knowledge,
) -> Result<String, UsherError> {
    let _ = (env, root, knowledge);
    Ok(String::new())
}

/// Hands one command and its arguments to its verb.
pub fn run(args: &[String], env: &InventoryEnv) -> Result<String, UsherError> {
    let _ = (args, env);
    Ok(String::new())
}
