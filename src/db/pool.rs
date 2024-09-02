use diesel::prelude::*;
use diesel_async::pooled_connection::bb8;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::AsyncPgConnection;
use futures_util::FutureExt;
use native_tls::Certificate;
use secrecy::{ExposeSecret, Secret};
use tokio_postgres::NoTls;

pub type PgConn = AsyncPgConnection;
pub type PgPooledConn<'a> = bb8::PooledConnection<'a, PgConn>;
pub type PgPool = bb8::Pool<PgConn>;

#[derive(Clone)]
pub struct DbPool(pub PgPool);

impl DbPool {
    #[allow(dead_code)]
    pub async fn get(&self) -> Result<PgPooledConn, bb8::RunError> {
        self.0.get().await
    }
}

pub async fn make_db_pool(database_url: &str) -> Result<DbPool, bb8::RunError> {
    let mut manager_config = ManagerConfig::<PgConn>::default();
    manager_config.custom_setup = Box::new(|url| establish(url).boxed());
    let manager =
        AsyncDieselConnectionManager::<PgConn>::new_with_config(database_url, manager_config);
    let pool = bb8::Pool::builder().build(manager).await?;
    Ok(DbPool(pool))
}

async fn establish(database_url: &str) -> ConnectionResult<AsyncPgConnection> {
    if database_url.contains("localhost") || database_url.contains("host.docker.internal") {
        let (client, connection) =
            tokio_postgres::connect(database_url, NoTls)
                .await
                .map_err(|e| {
                    ConnectionError::BadConnection(format!(
                        "Error connecting to {}: {}",
                        database_url, e
                    ))
                })?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("connection error: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    } else {
        let pg_cert = include_bytes!("../../pg_cert.pem");
        let certificate = Certificate::from_pem(pg_cert).map_err(|e| {
            ConnectionError::BadConnection(format!("Error reading certificate: {}", e))
        })?;
        let tls_connector = native_tls::TlsConnector::builder()
            .add_root_certificate(certificate)
            .build()
            .map_err(|e| {
                ConnectionError::BadConnection(format!("Error building TLS connector: {}", e))
            })?;
        let postgres_tls = postgres_native_tls::MakeTlsConnector::new(tls_connector);
        let (client, connection) = tokio_postgres::connect(database_url, postgres_tls)
            .await
            .map_err(|e| {
                ConnectionError::BadConnection(format!(
                    "Error connecting to {}: {}",
                    database_url, e
                ))
            })?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("Connection error: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    }
}

pub fn make_db_url(
    database_username: &str,
    database_password: &Secret<String>,
    database_host: &str,
    database_port: u16,
    database_name: &str,
) -> Secret<String> {
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database_username,
        database_password.expose_secret(),
        database_host,
        database_port,
        database_name
    );
    Secret::new(db_url)
}
