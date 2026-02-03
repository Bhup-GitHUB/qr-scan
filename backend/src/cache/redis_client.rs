use redis::aio::ConnectionManager;

pub async fn create_redis_client(redis_url: &str) -> redis::RedisResult<ConnectionManager> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_connection_manager().await?;
    Ok(conn)
}
