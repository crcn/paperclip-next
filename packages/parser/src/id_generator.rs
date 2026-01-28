use crc32fast::Hasher;

/// Generate document ID from file path using CRC32
pub fn get_document_id(path: &str) -> String {
    let mut buff = String::from(path);
    if !path.starts_with("file://") {
        buff = format!("file://{}", buff);
    }

    let mut hasher = Hasher::new();
    hasher.update(buff.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Sequential ID generator for AST nodes within a document
#[derive(Clone)]
pub struct IDGenerator {
    seed: String, // Document ID (CRC32)
    count: u32,   // Sequential counter
}

impl IDGenerator {
    pub fn new(path: &str) -> Self {
        Self {
            seed: get_document_id(path),
            count: 0,
        }
    }

    pub fn from_seed(seed: String) -> Self {
        Self { seed, count: 0 }
    }

    /// Generate next sequential ID
    pub fn new_id(&mut self) -> String {
        self.count += 1;
        format!("{}-{}", self.seed, self.count)
    }

    /// Get document ID seed
    pub fn seed(&self) -> &str {
        &self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_id_generation() {
        let id1 = get_document_id("/entry.pc");
        let id2 = get_document_id("/entry.pc");

        // Same path always generates same ID
        assert_eq!(id1, id2);

        // Different paths generate different IDs
        let id3 = get_document_id("/styles.pc");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_sequential_ids() {
        let mut gen = IDGenerator::new("/test.pc");

        let id1 = gen.new_id();
        let id2 = gen.new_id();
        let id3 = gen.new_id();

        // IDs are sequential
        assert!(id1.ends_with("-1"));
        assert!(id2.ends_with("-2"));
        assert!(id3.ends_with("-3"));

        // All share same seed
        let seed = gen.seed();
        assert!(id1.starts_with(seed));
        assert!(id2.starts_with(seed));
        assert!(id3.starts_with(seed));
    }
}
