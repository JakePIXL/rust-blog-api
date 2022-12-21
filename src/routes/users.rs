use actix_web::{HttpResponse, get, post, delete, put, Responder};
// use serde::Deserialize;

// use sea_orm::*;

// use entities::user::Entity as User;

// #[derive(Debug, Deserialize)]
// pub struct Params {
//     page: Option<u64>,
//     posts_per_page: Option<u64>,
// }

#[get("/")]
async fn get_all() -> impl Responder {
    HttpResponse::Ok().body("get all posts")
}

#[get("/{id}")]
async fn get_by_id() -> HttpResponse {
    HttpResponse::Ok().body("get post by id")
}

#[post("/")]
async fn create() -> impl Responder {
    HttpResponse::Ok().body("created post")
}

#[put("/{id}")]
async fn update() -> HttpResponse {
    HttpResponse::Ok().body("update")
}

#[delete("/{id}")]
async fn delete() -> HttpResponse {
    HttpResponse::Ok().body("delete")
}