use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// State for a pending conversation awaiting clarification
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are part of public API
pub struct PendingConversation {
    /// The original transcript that needs clarification
    pub transcript: String,
    /// The question asked to the user
    pub question: String,
    /// When this conversation was started
    pub created_at: DateTime<Utc>,
    /// Chat ID for this conversation
    pub chat_id: i64,
}

/// Manages conversation state for users awaiting clarification
#[derive(Debug, Clone)]
pub struct ConversationState {
    /// Map of chat_id -> pending conversation
    pending: Arc<RwLock<HashMap<i64, PendingConversation>>>,
    /// Timeout for pending conversations (default: 1 hour)
    timeout: Duration,
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            timeout: Duration::hours(1),
        }
    }

    /// Store a pending conversation awaiting clarification
    pub async fn set_pending(&self, chat_id: i64, transcript: String, question: String) {
        let conversation = PendingConversation {
            transcript,
            question,
            created_at: Utc::now(),
            chat_id,
        };

        let mut pending = self.pending.write().await;
        pending.insert(chat_id, conversation);
    }

    /// Get and remove a pending conversation if it exists and hasn't expired
    pub async fn take_pending(&self, chat_id: i64) -> Option<PendingConversation> {
        let mut pending = self.pending.write().await;

        if let Some(conversation) = pending.remove(&chat_id) {
            // Check if it's still valid (not expired)
            if Utc::now() - conversation.created_at < self.timeout {
                return Some(conversation);
            }
            // Expired, don't return it
            tracing::debug!("Pending conversation for {} expired", chat_id);
        }

        None
    }

    /// Check if there's a pending conversation for this chat
    #[allow(dead_code)] // Part of public API
    pub async fn has_pending(&self, chat_id: i64) -> bool {
        let pending = self.pending.read().await;

        if let Some(conversation) = pending.get(&chat_id) {
            // Check if it's still valid
            Utc::now() - conversation.created_at < self.timeout
        } else {
            false
        }
    }

    /// Clear a pending conversation without returning it
    #[allow(dead_code)] // Part of public API
    pub async fn clear_pending(&self, chat_id: i64) {
        let mut pending = self.pending.write().await;
        pending.remove(&chat_id);
    }

    /// Clean up expired conversations (call periodically)
    #[allow(dead_code)] // Part of public API
    pub async fn cleanup_expired(&self) {
        let mut pending = self.pending.write().await;
        let now = Utc::now();

        pending.retain(|chat_id, conv| {
            let expired = now - conv.created_at >= self.timeout;
            if expired {
                tracing::debug!("Cleaning up expired conversation for {}", chat_id);
            }
            !expired
        });
    }
}

impl Default for ConversationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set_and_take_pending() {
        let state = ConversationState::new();

        state
            .set_pending(
                12345,
                "John likes pizza".to_string(),
                "Which John?".to_string(),
            )
            .await;

        assert!(state.has_pending(12345).await);

        let pending = state.take_pending(12345).await;
        assert!(pending.is_some());
        let conv = pending.unwrap();
        assert_eq!(conv.transcript, "John likes pizza");
        assert_eq!(conv.question, "Which John?");
        assert_eq!(conv.chat_id, 12345);

        // Should no longer have pending after take
        assert!(!state.has_pending(12345).await);
    }

    #[tokio::test]
    async fn test_take_nonexistent() {
        let state = ConversationState::new();

        let pending = state.take_pending(99999).await;
        assert!(pending.is_none());
    }

    #[tokio::test]
    async fn test_has_pending_false_when_empty() {
        let state = ConversationState::new();
        assert!(!state.has_pending(12345).await);
    }

    #[tokio::test]
    async fn test_clear_pending() {
        let state = ConversationState::new();

        state
            .set_pending(12345, "test".to_string(), "question".to_string())
            .await;

        assert!(state.has_pending(12345).await);

        state.clear_pending(12345).await;

        assert!(!state.has_pending(12345).await);
    }

    #[tokio::test]
    async fn test_multiple_chats() {
        let state = ConversationState::new();

        state
            .set_pending(111, "transcript1".to_string(), "q1".to_string())
            .await;
        state
            .set_pending(222, "transcript2".to_string(), "q2".to_string())
            .await;

        assert!(state.has_pending(111).await);
        assert!(state.has_pending(222).await);

        let p1 = state.take_pending(111).await.unwrap();
        assert_eq!(p1.transcript, "transcript1");

        // 222 should still exist
        assert!(state.has_pending(222).await);
        assert!(!state.has_pending(111).await);
    }

    #[tokio::test]
    async fn test_overwrite_pending() {
        let state = ConversationState::new();

        state
            .set_pending(12345, "first".to_string(), "q1".to_string())
            .await;
        state
            .set_pending(12345, "second".to_string(), "q2".to_string())
            .await;

        let pending = state.take_pending(12345).await.unwrap();
        assert_eq!(pending.transcript, "second");
        assert_eq!(pending.question, "q2");
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let state = ConversationState::new();

        state
            .set_pending(12345, "test".to_string(), "q".to_string())
            .await;

        // Cleanup shouldn't remove fresh conversations
        state.cleanup_expired().await;
        assert!(state.has_pending(12345).await);
    }

    #[tokio::test]
    async fn test_default_impl() {
        let state = ConversationState::default();
        assert!(!state.has_pending(12345).await);
    }

    #[tokio::test]
    async fn test_clone_state() {
        let state = ConversationState::new();

        state
            .set_pending(12345, "test".to_string(), "q".to_string())
            .await;

        let cloned = state.clone();

        // Both should see the same pending conversation (Arc shared)
        assert!(cloned.has_pending(12345).await);
    }
}
