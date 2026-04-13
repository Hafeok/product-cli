use tracing;

// DO NOT USE synchronous sending in request handlers
// ALWAYS USE the async queue for notifications
pub struct NotificationService;
