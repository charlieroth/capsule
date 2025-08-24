use crate::jobs::{JobHandler, JobHandlerFactory};
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;

/// Registry of job handlers by kind
#[derive(Default)]
pub struct JobRegistry {
    handlers: HashMap<&'static str, JobHandlerFactory>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a job handler for a specific kind
    pub fn register<H: JobHandler + Clone + 'static>(&mut self, handler: H) {
        let kind = handler.kind();
        let factory: JobHandlerFactory =
            Box::new(move |_payload| Ok(Box::new(handler.clone()) as Box<dyn JobHandler>));
        self.handlers.insert(kind, factory);
    }

    /// Create a handler instance for the given job kind and payload
    pub fn create_handler(&self, kind: &str, payload: Value) -> Result<Box<dyn JobHandler>> {
        let factory = self
            .handlers
            .get(kind)
            .ok_or_else(|| anyhow!("No handler registered for job kind: {}", kind))?;

        factory(payload)
    }

    /// Get all registered job kinds
    pub fn registered_kinds(&self) -> Vec<&'static str> {
        self.handlers.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::JobHandler;
    use async_trait::async_trait;
    use serde_json::json;
    use sqlx::PgPool;
    use tracing::Span;

    #[derive(Clone)]
    struct TestJobHandler;

    #[async_trait]
    impl JobHandler for TestJobHandler {
        async fn run(&self, _payload: Value, _pool: &PgPool, _span: Span) -> anyhow::Result<()> {
            Ok(())
        }

        fn kind(&self) -> &'static str {
            "test_job"
        }
    }

    #[test]
    fn test_registry_registration() {
        let mut registry = JobRegistry::new();
        registry.register(TestJobHandler);

        let kinds = registry.registered_kinds();
        assert_eq!(kinds, vec!["test_job"]);
    }

    #[test]
    fn test_create_handler() {
        let mut registry = JobRegistry::new();
        registry.register(TestJobHandler);

        let result = registry.create_handler("test_job", json!({}));
        assert!(result.is_ok());

        let result = registry.create_handler("unknown_job", json!({}));
        assert!(result.is_err());
    }
}
