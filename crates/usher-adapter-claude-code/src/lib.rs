//! The Claude Code adapter (Story 1.5, FR-43): finds the settings sources of Claude
//! Code on a host and lists them through the source-listing port.
//!
//! Locations, layers, read rules and the version link come from harness knowledge,
//! not from this crate (AD-6). The harness is never run.

#![forbid(unsafe_code)]

pub mod version;

pub use version::{VersionReading, read_version};

use usher_core::{LaunchContext, SourceLister, SourceListing, SystemRoot, UsherError};
use usher_knowledge::{KnowledgeFile, Platform};

/// The Claude Code adapter: lists the settings sources knowledge names for the
/// harness, under the host's system root.
#[derive(Debug, Clone)]
pub struct ClaudeCodeAdapter {
    knowledge: KnowledgeFile,
    system_root: SystemRoot,
    platform: Platform,
}

impl ClaudeCodeAdapter {
    /// An adapter over `knowledge`. Production passes [`SystemRoot::host`] and the
    /// build's own platform; only tests pass others.
    pub fn new(knowledge: KnowledgeFile, system_root: SystemRoot, platform: Platform) -> Self {
        ClaudeCodeAdapter {
            knowledge,
            system_root,
            platform,
        }
    }
}

impl SourceLister for ClaudeCodeAdapter {
    fn list_sources(&self, context: &LaunchContext) -> Result<SourceListing, UsherError> {
        let _ = (&self.knowledge, &self.system_root, &self.platform, context);
        Ok(SourceListing {
            harness: String::new(),
            harness_version: None,
            knowledge_version: String::new(),
            sources: Vec::new(),
            absent_layers: Vec::new(),
            not_checked: Vec::new(),
        })
    }
}
