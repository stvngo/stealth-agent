pub mod capture;
pub mod transcribe;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub speaker: Speaker,
    pub text: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Speaker {
    Me,
    Interviewer,
    Unknown,
}

impl std::fmt::Display for Speaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Speaker::Me => write!(f, "Me"),
            Speaker::Interviewer => write!(f, "Interviewer"),
            Speaker::Unknown => write!(f, "Unknown"),
        }
    }
}
