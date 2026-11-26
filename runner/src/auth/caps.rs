// The actual capability

use rcan::Capability;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Ord, Serialize, Deserialize)]
pub enum Cap {
    All,
    Info,
    Issue,
    Revoke,
    Status,
    PathTest { path: String },
}

impl Capability for Cap {
    fn permits(&self, other: &Self) -> bool {
        match (self, other) {
            (Cap::All, _) => true,
            (Cap::Info, Cap::Info) => true,
            (Cap::Issue, Cap::Issue) => true,
            (Cap::Revoke, Cap::Revoke) => true,
            (Cap::Status, Cap::Status) => true,
            (Cap::PathTest { path }, Cap::PathTest { path: otherpath }) => {
                self.path_check(path, otherpath)
            }
            (_, _) => false,
        }
    }
}

impl Cap {
    fn path_check(&self, source: &String, other: &String) -> bool {
        if !source.starts_with("/") {
            return false;
        }
        if !other.starts_with("/") {
            return false;
        }
        if other.starts_with(source) {
            return true;
        }
        false
    }
}
