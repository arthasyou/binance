use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures_util::stream::Scan;

#[derive(Debug, Clone)]
pub struct SecretKey {
    pub id: String,
    pub api_key: String,
    pub api_secret: String,
}

impl SecretKey {
    pub fn new(id: String, api_key: String, api_secret: String) -> Self {
        SecretKey {
            id,
            api_key,
            api_secret,
        }
    }
}

pub struct KeyManager {
    keys: Mutex<HashMap<String, SecretKey>>, // 锁住 HashMap，确保多线程安全
}

impl KeyManager {
    // 创建新的 KeyManager，初始化 keys
    pub fn new() -> Arc<Self> {
        let map = HashMap::new(); // 初始化为空的 HashMap
        Arc::new(KeyManager {
            keys: Mutex::new(map),
        })
    }

    // 插入一个新密钥
    pub fn insert_key(&self, key: SecretKey) {
        let mut map = self.keys.lock().unwrap();
        map.insert(key.id.clone(), key);
    }

    // 删除一个密钥
    pub fn delete_key(&self, key_id: &str) {
        let mut map = self.keys.lock().unwrap();
        map.remove(key_id);
    }

    // 获取一个密钥
    pub fn get_key(&self, key_id: &str) -> Option<SecretKey> {
        let map = self.keys.lock().unwrap();
        map.get(key_id).cloned()
    }
}
