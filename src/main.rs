use std::{env, time::Duration};

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, web};
use api::{admin::admin, client::client};
use dotenvy::dotenv;
use migration::{
    Migrator, MigratorTrait,
    sea_orm::{ConnectOptions, Database, DatabaseConnection},
};
use redis::Client;

mod api;
mod bots;
mod state;
mod tasks;
mod util;

pub type Result<T> = crate::util::error::Result<T>;
pub type HttpResult = Result<HttpResponse>;

#[derive(Clone)]
pub struct AppState {
    mysql: DatabaseConnection,
    redis: Client,
}

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

use crate::tasks::{
    bsky_task::watch_merge_train_feed::spawn_merge_train_task,
    github_task::{
        watch_issue_list::get_new_issues, watch_milestones::spawn_milestone_task,
        watch_pr::get_new_commits,
    },
};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let mysql_url = env::var("DATABASE_URL").expect("请配置数据库链接");
    let redis_url = env::var("REDIS").expect("请配置Redis链接");
    let redis_client = redis::Client::open(redis_url).expect("连接Redis失败");

    let mut opt = ConnectOptions::new(mysql_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false)
        .sqlx_logging_level(log::LevelFilter::Error);

    let mysql_client = Database::connect(opt).await.expect("连接MYSQL数据库失败");

    // 运行数据库迁移
    Migrator::up(&mysql_client, None)
        .await
        .expect("数据库迁移失败");

    let app_state = AppState {
        mysql: mysql_client.clone(),
        redis: redis_client.clone(),
    };

    let web_app_state = web::Data::new(app_state.clone());

    // 异步任务
    get_new_issues().unwrap();
    get_new_commits().unwrap();
    spawn_milestone_task(app_state.clone()).unwrap();
    spawn_merge_train_task(app_state.clone());

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(web_app_state.clone())
            .service(web::scope("api").service(admin()).service(client()))
    })
    .bind(("127.0.0.1", 15698))?
    .run()
    .await
}
