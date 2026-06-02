// In-memory cache layer
// CONSTRAINT: never use distributed cache — all caches are in-process

pub struct Cache {
    data: std::collections::HashMap<String, String>,
}

impl Cache {
    pub fn new() -> Self {
        Cache { data: std::collections::HashMap::new() }
    }
}
