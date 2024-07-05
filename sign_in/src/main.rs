mod main_with_mysql;

use actix_web::{web, App, HttpServer, HttpResponse, Responder, HttpRequest};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use tokio_postgres::{ Error};
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use std::sync::{Arc, Mutex};
use serde_json::json;
use log::{info, error};
use bb8_postgres::{PostgresConnectionManager, tokio_postgres};
use bb8_postgres::bb8::{Pool, PooledConnection, RunError};
use postgres::NoTls;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    referral_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    token: String,
}

async fn register_user(user: web::Json<RegisterRequest>, pool: web::Data<ConPool>) -> impl Responder {
    let client = pool.get().await.unwrap();
    let hashed_password = hash(&user.password, 4).unwrap();
    let valid_referral_code = "1123";
    info!("Registering user: {}, referral_code is {}", &user.username, &user.referral_code);
    if user.referral_code != valid_referral_code {
        return HttpResponse::BadRequest().json(json!({"message": "Invalid referral code"}));
    }

    let statement = client.prepare("INSERT INTO users (username, password) VALUES ($1, $2)").await.unwrap();
    let result = client.execute(&statement, &[&user.username, &hashed_password]).await;

    match result {
        Ok(_) => {
            info!("User registered successfully: {}", &user.username);
            HttpResponse::Ok().json(json!({"message": "User registered successfully"}))
        }
        Err(err) => {
            error!("Error registering user: {}", err);
            HttpResponse::InternalServerError().json(json!({"message": "User registration failed"}))
        }
    }
}

async fn login_user(user: web::Json<User>, pool: web::Data<ConPool>) -> impl Responder {
    let client = pool.get().await.unwrap();
    let statement = client.prepare("SELECT password FROM users WHERE username = $1").await.unwrap();

    info!("Logging in user: {}", &user.username);

    if let Ok(rows) = client.query(&statement, &[&user.username]).await {
        if rows.is_empty() {
            return HttpResponse::Unauthorized().json(json!({"message": "Invalid username or password"}));
        }

        let stored_password: &str = rows[0].get(0);
        if verify(&user.password, stored_password).unwrap() {
            let my_claims = Claims { sub: user.username.clone(), exp: 10000000000 };
            let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();
            return HttpResponse::Ok().json(TokenResponse { token });
        }
    }
    HttpResponse::Unauthorized().json(json!({"message": "Invalid username or password"}))
}

async fn monitor_login(req: HttpRequest, pool: web::Data<ConPool>) -> impl Responder {
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                let validation = Validation::new(Algorithm::HS256);
                match decode::<Claims>(token, &DecodingKey::from_secret("secret".as_ref()), &validation) {
                    Ok(token_data) => {
                        let client = pool.get().await.unwrap();
                        let statement = client.prepare("UPDATE login_attempts SET attempts = attempts + 1 WHERE username = $1").await.unwrap();
                        let _ = client.execute(&statement, &[&token_data.claims.sub]).await;
                        return HttpResponse::Ok().json(json!({"message": "Login monitored"}));
                    }
                    Err(_) => return HttpResponse::Unauthorized().json(json!({"message": "Invalid token"})),
                }
            }
        }
    }
    HttpResponse::Unauthorized().json(json!({"message": "No token provided"}))
}

#[actix_web::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // let database_url = "postgres://postgres:123@localhost:5432/mydb";
    // let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
    //
    // let client = Arc::new(Mutex::new(client));
    //
    // tokio::spawn(async move {
    //     if let Err(e) = connection.await {
    //         eprintln!("connection error: {}", e);
    //     }
    // });

    let pool = get_pg_pool().await;
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
            )
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login_user))
            .route("/monitor", web::get().to(monitor_login))
    })
        .bind("0.0.0.0:8080").unwrap()
        .run()
        .await.unwrap();

    Ok(())
}

type ConPool = Arc<Pool<PostgresConnectionManager<NoTls>>>;

const POOL_SIZE: u32 = 15;

pub async fn get_pg_pool() -> ConPool {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config.host("localhost");
    pg_config.port(5432);
    pg_config.user("postgres");
    pg_config.password("123");
    pg_config.dbname("postgres");
    let manager = PostgresConnectionManager::new(pg_config, NoTls);
    let pool = Arc::new(Pool::builder().max_size(POOL_SIZE).build(manager).await.unwrap());

    pool
}