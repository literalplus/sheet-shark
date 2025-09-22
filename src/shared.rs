use tracing::warn;

pub mod summary;

pub const BREAK_PROJECT_KEY: &str = "x";

/// Simple local version tracker for saving with a single actor.
///
/// This does not assume that the remote stores version numbers and is suitable
/// for resetting the number for every edit session.
///
/// **Important:** This is NOT a db-side optimistic locking mechanism, but instead
/// serves to communicate to the frontend *which* of its changes have been saved.
/// This is relevant because the persistence layer might take some time to process
/// thelast save and the user might already trigger the next.
#[derive(Debug, Clone)]
pub struct DataVersion {
    /// version number of the local copy
    pub local: DataVersionNumber,
    /// version number of the last copy that was saved
    pub saved: DataVersionNumber,

    /// local version number that has been sent to be saved
    pub sent: Option<DataVersionNumber>,
}

pub type DataVersionNumber = i32;

impl DataVersion {
    pub fn fresh() -> Self {
        Self {
            local: 1,
            saved: 0,
            sent: None,
        }
    }

    pub fn loaded() -> Self {
        Self {
            local: 1,
            saved: 1,
            sent: None,
        }
    }

    pub fn touch(&mut self) {
        if self.local > self.saved { // already touched and not saved, combine these changes into one version
        } else {
            self.local += 1;
        }
    }

    pub fn mark_sent(&mut self) {
        // already guaranteed to be (non-strictly) monotonically increasing because `local` is
        self.sent = Some(self.local);
    }

    pub fn notify_saved(&mut self, saved_version: DataVersionNumber) {
        if saved_version <= self.saved {
            warn!(
                "DataVersion received out-of-order save notification {saved_version} - This shouldn't happen"
            );
            return;
        }
        self.saved = saved_version;
        if let Some(sent) = self.sent
            && sent == saved_version
        {
            self.sent = None;
        }
    }

    pub fn should_save(&self) -> bool {
        self.is_dirty() && self.sent != Some(self.local)
    }

    fn is_dirty(&self) -> bool {
        self.saved != self.local
    }
}
