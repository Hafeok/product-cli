use tracing;

// BY CONVENTION all middleware is tower-compatible
// MUST NOT log request bodies in production
pub struct LoggingLayer;
