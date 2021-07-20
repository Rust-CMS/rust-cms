pub mod config_models;
pub mod module_models;
pub mod page_models;

use actix_web::web;
use diesel::{MysqlConnection, r2d2::{ConnectionManager, Pool, PoolError, PooledConnection}};

use crate::{controllers::config_controllers::LocalConfig, middleware::errors_middleware::CustomHttpError};

pub type MySQLPool = Pool<ConnectionManager<MysqlConnection>>;
pub type MySQLPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

/// CRUD implementation.
pub trait Model<TQueryable, TMutable, TPrimary, TDto = TQueryable> {
    fn create(new: &TMutable, db: &MysqlConnection) -> Result<usize, diesel::result::Error>;
    fn read_one(id: TPrimary, db: &MysqlConnection) -> Result<TDto, diesel::result::Error>;
    fn read_all(db: &MysqlConnection) -> Result<Vec<TDto>, diesel::result::Error>;
    fn update(
        id: TPrimary,
        new: &TMutable,
        db: &MysqlConnection,
    ) -> Result<usize, diesel::result::Error>;
    fn delete(id: TPrimary, db: &MysqlConnection) -> Result<usize, diesel::result::Error>;
}

pub trait DTO<TColumns> {
    fn columns() -> TColumns;
}

/// Trait that enforces a  Model to be joinable if that is desired.
/// This should use associations rather than Left or Right join.
/// https://docs.diesel.rs/diesel/associations/index.html
pub trait Joinable<TLeft, TRight, TPrimary> {
    fn read_one_join_on(
        id: TPrimary,
        db: &MysqlConnection,
    ) -> Result<(TLeft, Vec<TRight>), diesel::result::Error>;
}

pub fn format_connection_string(conf: LocalConfig) -> String {
    format!(
        "mysql://{}:{}@{}:{}/{}",
        conf.mysql_username,
        conf.mysql_password,
        conf.mysql_url,
        conf.mysql_port,
        conf.mysql_database
    )
}

pub fn establish_database_connection(conf: LocalConfig) -> Option<MySQLPool> {
    let db_url = format_connection_string(conf);

    Some(init_pool(&db_url).expect("Failed to create pool."))
}

pub fn init_connection(db_url: &str) -> ConnectionManager<diesel::MysqlConnection> {
    ConnectionManager::<MysqlConnection>::new(db_url)
}

// https://dev.to/werner/practical-rust-web-development-connection-pool-46f4
pub fn init_pool(db_url: &str) -> Result<MySQLPool, PoolError> {
    let manager = init_connection(db_url);
    Pool::builder().max_size(2).build(manager)
}

pub fn pool_handler(pool: web::Data<MySQLPool>) -> Result<MySQLPooledConnection, CustomHttpError> {
    pool.get().or(Err(CustomHttpError::Unknown))
}
