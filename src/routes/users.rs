use actix_web::{web, HttpResponse, get, post, delete, put, Responder};
// use serde::Deserialize;

// use sea_orm::*;

// use entities::user::Entity as User;

// #[derive(Debug, Deserialize)]
// pub struct Params {
//     page: Option<u64>,
//     posts_per_page: Option<u64>,
// }

#[get("/users/")]
async fn get_all() -> impl Responder {
    HttpResponse::Ok().body("get all posts")
}

#[get("/users/{id}")]
async fn get_by_id() -> HttpResponse {
    HttpResponse::Ok().body("get post by id")
}

#[post("/users/")]
async fn create() -> impl Responder {
    HttpResponse::Ok().body("created post")
}

#[put("/users/{id}")]
async fn update() -> HttpResponse {
    HttpResponse::Ok().body("update")
}

#[delete("/users/{id}")]
async fn delete() -> HttpResponse {
    HttpResponse::Ok().body("delete")
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all);
    cfg.service(get_by_id);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}