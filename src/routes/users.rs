use actix_web::{web, HttpResponse, get, post, delete, put, Responder, HttpRequest};

use bcrypt::{hash, verify, DEFAULT_COST};

use jsonwebtoken::{Algorithm, EncodingKey, DecodingKey, decode, Validation, Header};
use jsonwebtoken::errors::Result as JwtResult;
use chrono::{Utc, Duration};

use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use sea_orm::*;

use entities::user::Entity as User;

use slugify::slugify;

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    users_per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
    keep_logged_in: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: i32,
    email: String,
    exp: i64,
}

fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

fn check_password(password: &str, hashed_password: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hashed_password)
}

fn create_jwt(user_id: i32, email: &str, days: i64) -> JwtResult<String> {
    let header = Header::new(Algorithm::HS256);

    let expiration_date = Utc::now() + Duration::days(days);


    let claims = Claims {
        user_id,
        email: email.to_string(),
        exp: expiration_date.timestamp(),
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or("secret".to_string());
    let key = EncodingKey::from_secret(secret.as_ref());

    jsonwebtoken::encode(&header, &claims, &key)
}

fn validate_token(token: &str) -> JwtResult<Claims> {
    let validation = Validation::new(Algorithm::HS256);
    let secret = std::env::var("JWT_SECRET").unwrap_or("secret".to_string());
    let key = DecodingKey::from_secret(secret.as_ref());

    let data = decode::<Claims>(token, &key, &validation)?;
    Ok(data.claims)
}

async fn check_is_valid(conn: &DatabaseConnection, user_id: i32, needs_admin: bool) -> bool {
    let user = entities::user::Entity::find()
        .filter(entities::user::Column::Id.eq(user_id))
        .one(conn)
        .await
        .expect("could not find user");

    match user {
        Some(user) => {
            if !user.is_active {
                return false;
            }

            if needs_admin && !user.is_admin {
                return false;
            }

            return true;
        }
        None => return false,
    }
}

#[get("/users/")]
async fn get_all(conn: web::Data<DatabaseConnection>, params: web::Query::<Params>, req: HttpRequest) -> impl Responder {

    let auth_header = req.headers().get("Authorization").unwrap().to_str().unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().body("invalid token");
    }

    let token = auth_header.split(" ").collect::<Vec<&str>>()[1];

    match validate_token(token) {
        Ok(data) => {
            if !check_is_valid(conn.as_ref(), data.user_id, true).await {
                return HttpResponse::Unauthorized().body("You are unauthorized to use this route.");
            }
            data
        },
        Err(e) => return HttpResponse::Unauthorized().body(format!("could not validate token: {}", e)),
    };

    let page = params.page.unwrap_or(1);
    let users_per_page = params.users_per_page.unwrap_or(10);


    let paginator = User::find()
        .order_by_asc(entities::post::Column::Id)
        .filter(entities::post::Column::IsPublished.eq(true))
        .paginate(conn.as_ref(), users_per_page);

    let num_pages = paginator.num_pages().await.unwrap();

    match paginator.fetch_page(page - 1).await {
        Ok(users) => {
            return HttpResponse::Ok().json((users, num_pages));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("could not fetch users: {}", e));
        }
    }
}

#[get("/users/{id}")]
async fn get_by_id(conn: web::Data<DatabaseConnection>, id: web::Path<i32>, req: HttpRequest) -> impl Responder {

    let auth_header = req.headers().get("Authorization").unwrap().to_str().unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().body("invalid token");
    }

    let token = auth_header.split(" ").collect::<Vec<&str>>()[1];

    match validate_token(token) {
        Ok(data) => {
            if !check_is_valid(conn.as_ref(), data.user_id, true).await {
                return HttpResponse::Unauthorized().body("You are unauthorized to use this route.");
            }
            data
        },
        Err(e) => return HttpResponse::Unauthorized().body(format!("could not validate token: {}", e)),
    };

    let user = User::find()
        .filter(entities::user::Column::Id.eq(id.clone()))
        .one(conn.as_ref())
        .await
        .expect("could not find post");

    match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => return HttpResponse::NotFound().body(format!("user with id: {} not found", id.clone())),
    }
}

#[post("/users/")]
async fn create(conn: web::Data<DatabaseConnection>, user_form: web::Form<entities::user::Model>) -> impl Responder {

    let hashed_password = hash_password(&user_form.password).unwrap();

    entities::user::ActiveModel {
        username: Set(slugify!(&user_form.username)),
        email: Set(user_form.email.clone()),
        password: Set(hashed_password),
        is_active: Set(false),
        is_admin: Set(false),
        ..Default::default()
    }
    .save(conn.as_ref())
    .await
    .unwrap();

    HttpResponse::Ok().body(format!("created user: {}", slugify!(&user_form.username)))
}

#[put("/users/{id}")]
async fn update(conn: web::Data<DatabaseConnection>, req: HttpRequest) -> impl Responder {

    let auth_header = req.headers().get("Authorization").unwrap().to_str().unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().body("invalid token");
    }

    let token = auth_header.split(" ").collect::<Vec<&str>>()[1];

    match validate_token(token) {
        Ok(data) => {
            if !check_is_valid(conn.as_ref(), data.user_id, true).await {
                return HttpResponse::Unauthorized().body("You are unauthorized to use this route.");
            }
            data
        },
        Err(e) => return HttpResponse::Unauthorized().body(format!("could not validate token: {}", e)),
    };
    
    HttpResponse::Ok().body("update")
}

#[delete("/users/{id}")]
async fn delete(conn: web::Data<DatabaseConnection>, req: HttpRequest) -> impl Responder {

    let auth_header = req.headers().get("Authorization").unwrap().to_str().unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().body("invalid token");
    }

    let token = auth_header.split(" ").collect::<Vec<&str>>()[1];

    match validate_token(token) {
        Ok(data) => {
            if !check_is_valid(conn.as_ref(), data.user_id, true).await {
                return HttpResponse::Unauthorized().body("You are unauthorized to use this route.");
            }
            data
        },
        Err(e) => return HttpResponse::Unauthorized().body(format!("could not validate token: {}", e)),
    };
    
    HttpResponse::Ok().body("delete")
}

#[post("/users/login")]
async fn login(conn: web::Data<DatabaseConnection>, login_form: web::Form<LoginForm>) -> HttpResponse {


    let username = login_form.username.clone();
    let password = login_form.password.clone();
    _ = login_form.keep_logged_in;

    let user = User::find()
        .filter(entities::user::Column::Username.eq(username.clone()))
        .one(conn.as_ref())
        .await
        .unwrap();

    match user {
        Some(user) => {
            if check_password(&password, &user.password).unwrap() {
                return HttpResponse::Ok().json(create_jwt(user.id, &user.email, 1).unwrap());
            } else {
                return HttpResponse::Unauthorized().body("Password is incorrect");
            }
        },
        None => {
            return HttpResponse::NotFound().body(format!("User {} not found", username));
        }

    }
}

#[post("/users/logout")]
async fn logout() -> HttpResponse {
    HttpResponse::Ok().body("logged out")
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all);
    cfg.service(get_by_id);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(login);
    cfg.service(logout);
}