// Portable persistence trait. Replaces direct `fs`/`dirs` calls so
// non-native frontends (web, FFI hosts) can supply their own backing store.
// Existing `load(config_dir)`/`save(config_dir)` methods on settings/
// campaign/endless/stats/achievements are untouched by this — this trait is
// additive groundwork for later phases (web `localStorage`, FFI hosts).

pub trait Storage {
    fn load(&self, namespace: &str, key: &str) -> Option<String>;
    fn save(&self, namespace: &str, key: &str, value: &str);
}

/// Native file-based `Storage`, backed by the same `dirs::config_dir()`
/// layout the existing `load`/`save` convenience methods already use.
#[cfg(feature = "native-storage")]
pub struct FsStorage;

#[cfg(feature = "native-storage")]
impl Storage for FsStorage {
    fn load(&self, namespace: &str, key: &str) -> Option<String> {
        let path = dirs::config_dir()?.join(namespace).join(key);
        std::fs::read_to_string(path).ok()
    }

    fn save(&self, namespace: &str, key: &str, value: &str) {
        let Some(dir) = dirs::config_dir() else { return };
        let path = dir.join(namespace).join(key);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// In-memory Storage for testing consumers of the trait without touching disk.
    #[derive(Default)]
    struct MemStorage(RefCell<HashMap<(String, String), String>>);

    impl Storage for MemStorage {
        fn load(&self, namespace: &str, key: &str) -> Option<String> {
            self.0.borrow().get(&(namespace.to_string(), key.to_string())).cloned()
        }

        fn save(&self, namespace: &str, key: &str, value: &str) {
            self.0.borrow_mut().insert((namespace.to_string(), key.to_string()), value.to_string());
        }
    }

    #[test]
    fn roundtrip_via_trait_object() {
        let storage: Box<dyn Storage> = Box::new(MemStorage::default());
        assert_eq!(storage.load("ns", "k"), None);
        storage.save("ns", "k", "value");
        assert_eq!(storage.load("ns", "k"), Some("value".to_string()));
    }

    #[test]
    fn namespaces_are_isolated() {
        let storage = MemStorage::default();
        storage.save("a", "k", "1");
        storage.save("b", "k", "2");
        assert_eq!(storage.load("a", "k"), Some("1".to_string()));
        assert_eq!(storage.load("b", "k"), Some("2".to_string()));
    }
}
