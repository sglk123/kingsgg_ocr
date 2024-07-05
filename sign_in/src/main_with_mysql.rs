use actix_web::{web, App, HttpServer, HttpResponse, Responder, HttpRequest};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPoolOptions;
use bcrypt::{hash, verify};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use std::sync::Arc;
use serde_json::json;
use log::{info, error};

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

// 配置类型
struct Config {
    mysql_uri: String,
}

// 全局连接池类型
type DbPool = Arc<sqlx::Pool<sqlx::MySql>>;

async fn get_mysql_pool(config: &Config) -> DbPool {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&config.mysql_uri)
        .await
        .expect("Failed to create pool.");
    Arc::new(pool)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = Config {
        mysql_uri: "mysql://sglk:123@localhost:3306/users".to_string(),
    };

    let pool = get_mysql_pool(&config).await;

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header().supports_credentials())
            .app_data(web::Data::new(pool.clone()))
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login_user))
            .route("/monitor", web::get().to(monitor_login))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

// 注册用户的函数示例
async fn register_user(user: web::Json<RegisterRequest>, pool: web::Data<DbPool>) -> impl Responder {
    println!("register_user received");
    let mut conn = pool.acquire().await.expect("Failed to get DB connection from pool");

    let hashed_password = hash(&user.password, 4).unwrap();
    let valid_referral_code = "1123";
    info!("Registering user: {}, referral_code is {}", &user.username, &user.referral_code);
    if user.referral_code != valid_referral_code {
        return HttpResponse::BadRequest().json(json!({"message": "Invalid referral code"}));
    }
    let query = "INSERT INTO users (username, password) VALUES (?, ?)";
    let result = sqlx::query(query)
        .bind(&user.username)
        .bind(hashed_password)
        .execute(&mut conn)
        .await;

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

// 登录用户的函数示例
async fn login_user(user: web::Json<User>, pool: web::Data<DbPool>) -> impl Responder {
    let username = &user.username;
    let password = &user.password;

    // Get a connection from the pool
    let mut conn = pool.acquire().await.expect("Failed to acquire a connection");

    // Query to check the username and get the hashed password
    let query = "SELECT password FROM users WHERE username = ?";
    match sqlx::query_as::<_, (String,)>(query) // This should map to the structure of the expected return
        .bind(username)
        .fetch_optional(&mut conn)
        .await {
        Ok(Some((db_password,))) => {
            if verify(password, &db_password).unwrap() {
                let my_claims = Claims { sub: username.clone(), exp: 10000000000 };
                let token = encode(&Header::default(), &my_claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();
                HttpResponse::Ok().json(TokenResponse { token })
            } else {
                HttpResponse::Unauthorized().json(json!({"message": "Invalid username or password"}))
            }
        },
        Ok(None) => HttpResponse::Unauthorized().json(json!({"message": "Invalid username or password"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"message": "Database error"}))
    }
}


async fn monitor_login(req: HttpRequest, pool: web::Data<DbPool>) -> impl Responder {

    HttpResponse::Ok().json("Login monitored")
}
