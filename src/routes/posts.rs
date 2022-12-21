use actix_web::{web, HttpResponse, get, post, delete, patch, Responder, HttpRequest};
// use actix_web_httpauth::headers::authorization::Authorization;
use serde::{Deserialize, Serialize};

use jsonwebtoken::{Algorithm, DecodingKey, decode, Validation};
use jsonwebtoken::errors::Result as JwtResult;

use sea_orm::*;

use entities::post::Entity as Post;
use slugify::slugify;

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: i32,
    email: String,
    exp: i64,
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

#[get("/posts/")]
async fn get_all(conn: web::Data<DatabaseConnection>, params: web::Query::<Params>) -> impl Responder {

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(10);


    let paginator = Post::find()
        .order_by_asc(entities::post::Column::Id)
        .filter(entities::post::Column::IsPublished.eq(true))
        .paginate(conn.as_ref(), posts_per_page);

    let num_pages = paginator.num_pages().await.unwrap();

    match paginator.fetch_page(page - 1).await {
        Ok(posts) => {
            return HttpResponse::Ok().json((posts, num_pages));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("could not fetch posts: {}", e));
        }
    }
}

#[get("/posts/{id}")]
async fn get_by_id(conn: web::Data<DatabaseConnection>, id: web::Path<i32>) -> HttpResponse {

    let post = Post::find()
        .filter(entities::post::Column::Id.eq(id.clone()))
        .one(conn.as_ref())
        .await
        .expect("could not find post");

    match post {
        Some(post) => HttpResponse::Ok().json(post),
        None => return HttpResponse::NotFound().body(format!("post with id: {} not found", id.clone())),
    }
}

#[get("/posts/{slug}")]
async fn get_by_slug(conn: web::Data<DatabaseConnection>, slug: web::Path<String>) -> HttpResponse {

    let post = Post::find()
        .filter(entities::post::Column::Slug.eq(slug.clone()))
        .one(conn.as_ref())
        .await
        .expect("could not find post");

    match post {
        Some(post) => HttpResponse::Ok().json(post),
        None => return HttpResponse::NotFound().body(format!("post with slug: {} not found", slug.clone())),
    }
}

#[post("/posts/")]
async fn create(conn: web::Data<DatabaseConnection>, post_form: web::Form<entities::post::Model>, req: HttpRequest) -> impl Responder {

    let auth_header = req.headers().get("Authorization").unwrap().to_str().unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().body("invalid token");
    }

    let token = auth_header.split(" ").collect::<Vec<&str>>()[1];

    let user = match validate_token(token) {
        Ok(data) => {
            if !check_is_valid(conn.as_ref(), data.user_id, true).await {
                return HttpResponse::Unauthorized().body("You are unauthorized to use this route.");
            }
            data
        },
        Err(e) => return HttpResponse::Unauthorized().body(format!("could not validate token: {}", e)),
    };


    match Post::find()
        .filter(entities::post::Column::Slug.eq(slugify!(&post_form.title, max_length = 20)))
        .one(conn.as_ref())
        .await
        .expect("could not find post")
    {
        Some(_) => return HttpResponse::BadRequest().body(format!("post with slug {} already exists", slugify!(&post_form.title, max_length = 20))),
        None => (),
    }

    entities::post::ActiveModel {
        slug: Set({
            if post_form.slug.is_none() {
                Some(slugify!(&post_form.title, max_length = 20))
            } else {
                post_form.slug.clone()
            }
        }),
        title: Set(post_form.title.clone()),
        text: Set(post_form.text.clone()),
        user_id: Set(Some(user.user_id.clone())),
        is_published: Set(post_form.is_published.clone()),
        ..Default::default()
    }
    .save(conn.as_ref())
    .await
    .expect("could not insert post");

    HttpResponse::Ok().body("created post")
}

#[patch("/posts/{id}")]
async fn update(conn: web::Data<DatabaseConnection>, id: web::Path<i32>, post_form: web::Form<entities::post::Model>, req: HttpRequest) -> impl Responder {

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

    let post = Post::find()
        .filter(entities::post::Column::Id.eq(id.clone()))
        .one(conn.as_ref())
        .await
        .expect("could not find post");

    match post {
        Some(post) => {
            let updated_post = entities::post::ActiveModel {
                id: Set(post.id.clone()),
                slug: Set({
                    if post_form.slug.is_none() {
                        Some(slugify!(&post_form.title, max_length = 20))
                    } else {
                        Some(slugify!(&post_form.slug.clone().unwrap(), max_length = 20))
                    }
                }),
                title: Set(post_form.title.clone()),
                text: Set(post_form.text.clone()),
                is_published: Set(post_form.is_published.clone()),
                ..Default::default()
            };

            return HttpResponse::Ok().json(updated_post.update(conn.as_ref()).await.expect("could not update post"));
        }
        None => return HttpResponse::NotFound().body(format!("post with id: {} not found", id.clone())),
    }
}

#[delete("/posts/{id}")]
async fn delete(conn: web::Data<DatabaseConnection>, id: web::Path<i32>, req: HttpRequest) -> impl Responder {

    let auth_header = req.headers().get("Authorization").unwrap().to_str().unwrap_or("");

    if !auth_header.starts_with("Bearer ") {
        return HttpResponse::Unauthorized().body("invalid token");
    }

    let token = auth_header.split(" ").collect::<Vec<&str>>()[1];

    let user = match validate_token(token) {
        Ok(data) => {
            if !check_is_valid(conn.as_ref(), data.user_id, true).await {
                return HttpResponse::Unauthorized().body("You are unauthorized to use this route.");
            }
            data
        },
        Err(e) => return HttpResponse::Unauthorized().body(format!("could not validate token: {}", e)),
    };    

    let found_post = Post::find()
        .filter(entities::post::Column::Id.eq(id.clone()))
        .one(conn.as_ref())
        .await
        .expect("could not find post");


    match found_post {
        Some(post) => {

            if post.user_id.unwrap() != user.user_id {
                return HttpResponse::Unauthorized().body("user is not authorized to delete this post");
            }

            entities::post::ActiveModel {
                id: Set(id.clone()),
                ..Default::default()
            };
        
            post.delete(conn.as_ref()).await.expect("could not delete post");

            return HttpResponse::Ok().body(format!("Deleted post: {}", id.clone()));
        }
        None => return HttpResponse::NotFound().body(format!("post with id: {} not found", id.clone())),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all);
    cfg.service(get_by_id);
    cfg.service(get_by_slug);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}