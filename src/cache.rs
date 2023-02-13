use std::collections::HashMap;

#[derive(Debug)]
pub struct Cache<T: Clone> {
    store: HashMap<String, T> 
}

impl<T: Clone> Default for Cache<T> {
    fn default() -> Self {
        Cache {
            store: HashMap::new()
        }
    }
}

impl<T: Clone> Cache<T> {
    pub fn get(&self, id: &str) -> Option<T> {
        if let Some(x) = self.store.get(id) {
            log::debug!("Found {} in cache", id);
            return Some(x.clone());
        };
        None
    } 

    pub fn put(& mut self, id: &str, v: &T) {
        self.store.insert(id.to_string(), v.clone());
    }
}
