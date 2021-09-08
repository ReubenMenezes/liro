use crate::config;
use mobc_redis::{
    redis::{self, AsyncCommands, FromRedisValue},
    RedisConnectionManager,
};
use thiserror::Error;

pub type Pool = mobc::Pool<RedisConnectionManager>;
pub type Connection = mobc::Connection<RedisConnectionManager>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("client initialization error: {0}")]
    ClientError(redis::RedisError),
    #[error("unable to get connection from pool: {0}")]
    PoolError(#[from] mobc::Error<redis::RedisError>),
    #[error("error while running command: {0}")]
    CommandError(#[from] redis::RedisError),
    #[error("type conversion error: {0}")]
    TypeError(redis::RedisError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn connect() -> Result<Pool> {
    let redis_uri = config::redis_uri();
    let client = redis::Client::open(redis_uri).map_err(|e| Error::ClientError(e))?;

    let manager = RedisConnectionManager::new(client);
    let pool = mobc::Pool::builder().max_open(20).build(manager);

    Ok(pool)
}

async fn get_connection(pool: &Pool) -> Result<Connection> {
    Ok(pool.get().await?)
}

pub async fn set(pool: &Pool, key: &str, value: &str) -> Result<()> {
    let mut conn = get_connection(&pool).await?;

    conn.set(key, value).await?;
    Ok(())
}

pub async fn get(pool: &Pool, key: &str) -> Result<String> {
    let mut conn = get_connection(&pool).await?;

    let value = conn.get(key).await?;
    Ok(FromRedisValue::from_redis_value(&value).map_err(|e| Error::TypeError(e))?)
}
