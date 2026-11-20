//! The probe runner: find the harness, check the control, then the claim.

use std::{
    cell::Cell,
    ffi::OsStr,
    fmt, io,
    path::{Path, PathBuf},
    process::Output,
    time::Duration,
};

use usher_testkit::{Probe, ScratchHome, probe};

use crate::{
    fragment::{Date, Fragment, Network, Outcome, bare_version},
    network::host_state,
};

/// Why a claim could not be measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unmeasured {
    /// The claim needs credentials the probe must not have.
    NeedsCredentials,
    /// The claim needs a network the probe runs without.
    NeedsNetwork,
    /// The run broke; the text says how.
    Broken(String),
}

/// A probe that could not run. It is never a pass and never a refutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRun {
    kind: Unmeasured,
    text: String,
}

/// What a probe is about and where its harness is looked for.
#[derive(Debug, Clone, Copy)]
pub struct Spec<'a> {
    /// The probe's name.
    pub name: &'a str,
    /// The harness binary name.
    pub binary: &'a str,
    /// Where to look for it; `PATH` when `None`.
    pub search_path: Option<&'a OsStr>,
    /// The localhost ports a run may reach; none by default.
    pub localhost_ports: &'a [u16],
    /// How long a started process may run before it is killed.
    pub timeout: Duration,
}

/// The harness a probe runs against.
#[derive(Debug)]
pub struct Harness {
    /// Where the binary was found.
    pub path: PathBuf,
    /// Its bare version number.
    pub version: String,
}

/// The only way a probe starts a process.
#[derive(Debug)]
pub struct Launcher<'a> {
    scratch: &'a ScratchHome,
    starts: Cell<usize>,
}

impl<'a> Spec<'a> {
    /// A spec with no search path, no localhost ports and the default timeout.
    pub fn new(name: &'a str, binary: &'a str) -> Spec<'a> {
        Spec {
            name,
            binary,
            search_path: None,
            localhost_ports: &[],
            timeout: Duration::ZERO,
        }
    }

    /// Looks for the binary on `path` instead of `PATH`.
    pub fn search_path(mut self, path: &'a OsStr) -> Spec<'a> {
        self.search_path = Some(path);
        self
    }

    /// Allows the run to reach exactly these localhost ports.
    pub fn localhost_ports(mut self, ports: &'a [u16]) -> Spec<'a> {
        self.localhost_ports = ports;
        self
    }

    /// Kills a started process that outlives `timeout`.
    pub fn timeout(mut self, timeout: Duration) -> Spec<'a> {
        self.timeout = timeout;
        self
    }
}

impl Launcher<'_> {
    /// Starts `program` with `args` and returns its output.
    pub fn run(&self, program: impl AsRef<OsStr>, args: &[&str]) -> io::Result<Output> {
        self.starts.set(self.starts.get() + 1);
        self.scratch.command(program).args(args).output()
    }

    /// How many processes this launcher has started.
    pub fn starts(&self) -> usize {
        self.starts.get()
    }

    /// The scratch home's root, for a probe that writes its own files.
    pub fn scratch_root(&self) -> &Path {
        self.scratch.root()
    }
}

impl fmt::Display for Unmeasured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unmeasured::NeedsCredentials => f.write_str("needs credentials"),
            Unmeasured::NeedsNetwork => f.write_str("needs network"),
            Unmeasured::Broken(text) => f.write_str(text),
        }
    }
}

impl NotRun {
    fn new(kind: Unmeasured) -> NotRun {
        let text = kind.to_string();
        NotRun { kind, text }
    }

    /// The kind of reason.
    pub fn kind(&self) -> &Unmeasured {
        &self.kind
    }

    /// The reason, without the `not run:` prefix.
    pub fn reason(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for NotRun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not run: {}", self.text)
    }
}

/// Runs one probe. The control runs first; when it fails, the claim is not run
/// and the probe is `NotRun`. A claim that holds is `Passed`, one that does not
/// is `Failed`, and one that cannot be measured is `NotRun`.
pub fn run<C, K>(
    spec: &Spec<'_>,
    scratch: &ScratchHome,
    today: Date,
    control: C,
    claim: K,
) -> Result<Fragment, NotRun>
where
    C: FnOnce(&Harness, &Launcher<'_>) -> Result<(), String>,
    K: FnOnce(&Harness, &Launcher<'_>) -> Result<bool, Unmeasured>,
{
    run_with_host_state(host_state(), spec, scratch, today, control, claim)
}

/// As [`run`], on a host that is taken to deny the network or not. This is the
/// seam a test uses to check what the fragment says on a host that cannot.
pub fn run_with_host_state<C, K>(
    host: Network,
    spec: &Spec<'_>,
    scratch: &ScratchHome,
    today: Date,
    control: C,
    claim: K,
) -> Result<Fragment, NotRun>
where
    C: FnOnce(&Harness, &Launcher<'_>) -> Result<(), String>,
    K: FnOnce(&Harness, &Launcher<'_>) -> Result<bool, Unmeasured>,
{
    let (path, version_line) = match probe(spec.binary, spec.search_path, scratch) {
        Probe::Found { path, version } => (path, version),
        Probe::Missing { reason } => return Err(NotRun::new(Unmeasured::Broken(reason))),
    };
    let Some(version) = bare_version(&version_line) else {
        return Err(NotRun::new(Unmeasured::Broken(format!(
            "`{} --version` printed no version number",
            spec.binary
        ))));
    };
    let harness = Harness { path, version };
    let launcher = Launcher {
        scratch,
        starts: Cell::new(0),
    };

    control(&harness, &launcher)
        .map_err(|text| NotRun::new(Unmeasured::Broken(format!("the control failed: {text}"))))?;
    let held = claim(&harness, &launcher).map_err(NotRun::new)?;

    let outcome = if held {
        Outcome::Passed
    } else {
        Outcome::Failed
    };
    Fragment::new(&harness.version, outcome, today, host)
        .map_err(|err| NotRun::new(Unmeasured::Broken(err.to_string())))
}

/// Runs one probe and panics with the `not run:` message when it cannot run,
/// so a maintainer's `--ignored` run never reads an unrun probe as ok. The
/// fragment is printed on stdout.
pub fn run_or_panic<C, K>(
    spec: &Spec<'_>,
    scratch: &ScratchHome,
    today: Date,
    control: C,
    claim: K,
) -> Fragment
where
    C: FnOnce(&Harness, &Launcher<'_>) -> Result<(), String>,
    K: FnOnce(&Harness, &Launcher<'_>) -> Result<bool, Unmeasured>,
{
    match run(spec, scratch, today, control, claim) {
        Ok(fragment) => {
            println!("probe {}: {}", spec.name, fragment.render());
            fragment
        }
        Err(not_run) => panic!("{not_run}"),
    }
}
