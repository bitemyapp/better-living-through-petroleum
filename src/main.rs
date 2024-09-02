use actix_web::{error, get, middleware, post, web, App, HttpResponse, HttpServer, Responder};

mod db;

use db::*;
use newtypes::UserId;

// Actix example based on https://github.com/actix/examples/tree/a31e6731d21340ac5e4d411e3db76a93f628a74e/databases/diesel
#[get("/user/{user_id}")]
async fn get_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<UserId>,
) -> actix_web::Result<impl Responder> {
    let user_id = user_id.into_inner();
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let user = db::models::User::find_user_by_uid(&mut conn, user_id).await.map_err(error::ErrorInternalServerError)?;

    Ok(match user {
        // user was found; return 200 response with JSON formatted user object
        Some(user) => HttpResponse::Ok().json(user),

        // user was not found; return 404 response with error message
        None => HttpResponse::NotFound().body(format!("No user found with UID: {user_id:?}")),
    })
}

#[post("/user")]
async fn add_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let user = db::models::NewUser::insert_new_user(&mut conn, &form.name).await.map_err(error::ErrorInternalServerError)?;
    // user was added successfully; return 201 response with new user info
    Ok(HttpResponse::Created().json(user))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // initialize DB pool outside of `HttpServer::new` so that it is shared across all workers
    let pool = db::make_db_pool(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")).await.expect("Failed to create pool");

    // log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(pool.clone()))
            // add request logger middleware
            .wrap(middleware::Logger::default())
            // add route handlers
            .service(get_user)
            .service(add_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
