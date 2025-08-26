//! Progress reporting and notification system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

/// Progress token for tracking long-running operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProgressToken(pub String);

impl ProgressToken {
    /// Create a new progress token with a unique identifier
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create a progress token from a string
    #[must_use]
    pub const fn from_string(token: String) -> Self {
        Self(token)
    }

    /// Get the token string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ProgressToken {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProgressToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Progress information for an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Current progress value
    pub progress: f64,
    /// Total expected progress (optional)
    pub total: Option<f64>,
    /// Human-readable message about current status
    pub message: Option<String>,
    /// Metadata associated with this progress update
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp of this progress update
    pub timestamp: SystemTime,
}

impl Progress {
    /// Create a new progress report
    #[must_use]
    pub fn new(progress: f64) -> Self {
        Self {
            progress,
            total: None,
            message: None,
            metadata: HashMap::new(),
            timestamp: SystemTime::now(),
        }
    }

    /// Set the total expected progress
    #[must_use]
    pub const fn with_total(mut self, total: f64) -> Self {
        self.total = Some(total);
        self
    }

    /// Set a status message
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Add metadata
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }

    /// Calculate progress percentage (0.0 to 100.0)
    #[must_use]
    pub fn percentage(&self) -> f64 {
        match self.total {
            Some(total) if total > 0.0 => (self.progress / total * 100.0).min(100.0),
            _ => self.progress.min(100.0),
        }
    }

    /// Check if operation is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        match self.total {
            Some(total) => self.progress >= total,
            None => self.progress >= 100.0,
        }
    }
}

/// Progress notification that can be sent to MCP clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    /// Progress token identifying the operation
    pub progress_token: ProgressToken,
    /// Progress information
    pub progress: Progress,
    /// Optional operation ID for grouping related operations
    pub operation_id: Option<String>,
}

/// Progress manager for tracking multiple operations
#[derive(Debug)]
pub struct ProgressManager {
    /// Active progress trackers
    trackers: Arc<std::sync::RwLock<HashMap<ProgressToken, ProgressTracker>>>,
    /// Global counter for generating operation IDs
    operation_counter: AtomicU64,
}

impl ProgressManager {
    /// Create a new progress manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            trackers: Arc::new(std::sync::RwLock::new(HashMap::new())),
            operation_counter: AtomicU64::new(0),
        }
    }

    /// Start tracking progress for a new operation
    pub fn start_operation<S: Into<String>>(&self, description: S) -> ProgressToken {
        let token = ProgressToken::new();
        let operation_id = self.operation_counter.fetch_add(1, Ordering::Relaxed);

        let tracker = ProgressTracker::new(
            token.clone(),
            description.into(),
            format!("op_{operation_id}"),
        );

        self.trackers
            .write()
            .unwrap()
            .insert(token.clone(), tracker);
        token
    }

    /// Update progress for an operation
    pub fn update_progress(
        &self,
        token: &ProgressToken,
        progress: f64,
        total: Option<f64>,
    ) -> crate::McpResult<()> {
        let mut trackers = self.trackers.write().unwrap();

        if let Some(tracker) = trackers.get_mut(token) {
            tracker.update_progress(progress, total);

            // Send notification to MCP client via notification system
            tracing::debug!(
                "Progress update: {} - {:.1}%",
                token,
                tracker.current_progress().percentage()
            );

            Ok(())
        } else {
            Err(crate::McpError::Tool(format!(
                "Progress token not found: {token}"
            )))
        }
    }

    /// Update progress with a message
    pub fn update_progress_with_message(
        &self,
        token: &ProgressToken,
        progress: f64,
        total: Option<f64>,
        message: String,
    ) -> crate::McpResult<()> {
        let mut trackers = self.trackers.write().unwrap();

        if let Some(tracker) = trackers.get_mut(token) {
            let progress_info = Progress::new(progress)
                .with_total(total.unwrap_or(100.0))
                .with_message(message);

            tracker.update_progress_full(progress_info);

            tracing::debug!(
                "Progress update: {} - {:.1}% - {}",
                token,
                tracker.current_progress().percentage(),
                tracker.current_progress().message.as_deref().unwrap_or("")
            );

            Ok(())
        } else {
            Err(crate::McpError::Tool(format!(
                "Progress token not found: {token}"
            )))
        }
    }

    /// Complete an operation
    pub fn complete_operation(&self, token: &ProgressToken) -> crate::McpResult<()> {
        let mut trackers = self.trackers.write().unwrap();

        if let Some(mut tracker) = trackers.remove(token) {
            tracker.complete();

            tracing::info!("Operation completed: {}", token);

            // Send completion notification to MCP client via notification system
            Ok(())
        } else {
            Err(crate::McpError::Tool(format!(
                "Progress token not found: {token}"
            )))
        }
    }

    /// Get current progress for an operation
    pub fn get_progress(&self, token: &ProgressToken) -> Option<Progress> {
        let trackers = self.trackers.read().unwrap();
        trackers.get(token).map(|t| t.current_progress().clone())
    }

    /// List all active operations
    pub fn active_operations(&self) -> Vec<(ProgressToken, Progress)> {
        let trackers = self.trackers.read().unwrap();
        trackers
            .iter()
            .map(|(token, tracker)| (token.clone(), tracker.current_progress().clone()))
            .collect()
    }

    /// Clean up completed operations older than the specified duration
    pub fn cleanup_old_operations(&self, max_age: Duration) {
        let mut trackers = self.trackers.write().unwrap();
        let now = SystemTime::now();

        trackers.retain(|_token, tracker| {
            if let Ok(age) = now.duration_since(tracker.current_progress().timestamp) {
                age < max_age
            } else {
                true // Keep if timestamp is in future (shouldn't happen)
            }
        });
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual progress tracker for a single operation
#[derive(Debug, Clone)]
struct ProgressTracker {
    #[allow(dead_code)]
    token: ProgressToken,
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    operation_id: String,
    progress: Progress,
    #[allow(dead_code)]
    started_at: SystemTime,
}

impl ProgressTracker {
    fn new(token: ProgressToken, description: String, operation_id: String) -> Self {
        Self {
            token,
            description,
            operation_id,
            progress: Progress::new(0.0),
            started_at: SystemTime::now(),
        }
    }

    fn update_progress(&mut self, progress: f64, total: Option<f64>) {
        self.progress.progress = progress;
        if let Some(t) = total {
            self.progress.total = Some(t);
        }
        self.progress.timestamp = SystemTime::now();
    }

    fn update_progress_full(&mut self, progress: Progress) {
        self.progress = progress;
    }

    fn complete(&mut self) {
        let total = self.progress.total.unwrap_or(100.0);
        self.progress.progress = total;
        self.progress.timestamp = SystemTime::now();
        self.progress.message = Some("Operation completed".to_string());
    }

    const fn current_progress(&self) -> &Progress {
        &self.progress
    }

    #[allow(dead_code)]
    fn elapsed(&self) -> Duration {
        self.started_at.elapsed().unwrap_or(Duration::from_secs(0))
    }
}

/// Global progress manager instance
static GLOBAL_PROGRESS_MANAGER: once_cell::sync::Lazy<ProgressManager> =
    once_cell::sync::Lazy::new(ProgressManager::new);

/// Get the global progress manager instance
#[must_use]
pub fn global_progress_manager() -> &'static ProgressManager {
    &GLOBAL_PROGRESS_MANAGER
}

/// Start tracking a new operation (convenience function)
pub fn start_progress<S: Into<String>>(description: S) -> ProgressToken {
    global_progress_manager().start_operation(description)
}

/// Update progress (convenience function)
pub fn update_progress(
    token: &ProgressToken,
    progress: f64,
    total: Option<f64>,
) -> crate::McpResult<()> {
    global_progress_manager().update_progress(token, progress, total)
}

/// Complete an operation (convenience function)
pub fn complete_progress(token: &ProgressToken) -> crate::McpResult<()> {
    global_progress_manager().complete_operation(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_progress_token() {
        let token1 = ProgressToken::new();
        let token2 = ProgressToken::new();

        assert_ne!(token1, token2);
        assert!(!token1.as_str().is_empty());
    }

    #[test]
    fn test_progress_calculation() {
        let progress = Progress::new(25.0).with_total(100.0);
        assert_eq!(progress.percentage(), 25.0);
        assert!(!progress.is_complete());

        let complete_progress = Progress::new(100.0).with_total(100.0);
        assert!(complete_progress.is_complete());
    }

    #[test]
    fn test_progress_manager() {
        let manager = ProgressManager::new();

        let token = manager.start_operation("test operation");
        assert!(manager.get_progress(&token).is_some());

        manager.update_progress(&token, 50.0, Some(100.0)).unwrap();

        let progress = manager.get_progress(&token).unwrap();
        assert_eq!(progress.progress, 50.0);
        assert_eq!(progress.total, Some(100.0));

        manager.complete_operation(&token).unwrap();
        assert!(manager.get_progress(&token).is_none());
    }

    #[tokio::test]
    async fn test_async_progress_tracking() {
        let manager = ProgressManager::new();
        let token = manager.start_operation("async operation");

        // Simulate async work with progress updates
        for i in 0..=10 {
            let progress = f64::from(i) * 10.0;
            manager
                .update_progress(&token, progress, Some(100.0))
                .unwrap();

            if i < 10 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        let final_progress = manager.get_progress(&token).unwrap();
        assert!(final_progress.is_complete());

        manager.complete_operation(&token).unwrap();
    }
}

// Add uuid dependency
// This would need to be added to Cargo.toml
// uuid = { version = "1.0", features = ["v4"] }
