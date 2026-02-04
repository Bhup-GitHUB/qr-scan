use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RedisClient {
    conn: Arc<Mutex<ConnectionManager>>,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_connection_manager().await?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        let mut conn = self.conn.lock().await;
        let value: Option<String> = conn.get(key).await.map_err(|e| e.to_string())?;

        match value {
            Some(raw) => {
                let parsed = serde_json::from_str::<T>(&raw).map_err(|e| e.to_string())?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, expiry_secs: usize) -> Result<(), String> {
        let serialized = serde_json::to_string(value).map_err(|e| e.to_string())?;
        let mut conn = self.conn.lock().await;
        conn.set_ex(key, serialized, expiry_secs as u64)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete(&self, key: &str) -> Result<(), String> {
        let mut conn = self.conn.lock().await;
        conn.del(key).await.map_err(|e| e.to_string())
    }
}
