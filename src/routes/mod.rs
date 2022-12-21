use actix_web::{web, HttpResponse, get, post, delete, put, Responder};
use serde::Deserialize;

use sea_orm::*;

use entities::post::Entity as Post;

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    posts_per_page: Option<u64>,
}

#[get("/")]
async fn get_all(conn: web::Data<DatabaseConnection>, params: web::Query::<Params>) -> impl Responder {

    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(10);


    let paginator = Post::find()
        .order_by_asc(entities::post::Column::Id)
        .paginate(conn.as_ref(), posts_per_page);

    let num_pages = paginator.num_pages().await.unwrap();

    match paginator.fetch_page(page).await.map(|p| (p, num_pages)) {
        Ok((posts, num_pages)) => {
            return HttpResponse::Ok().json((posts, num_pages));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("could not fetch posts: {}", e));
        }
    }
}

#[get("/{id}")]
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

#[post("/")]
async fn create(conn: web::Data<DatabaseConnection>, post_form: web::Form<entities::post::Model>) -> impl Responder {

    entities::post::ActiveModel {
        title: Set(post_form.title.clone()),
        text: Set(post_form.text.clone()),
        user_id: Set(post_form.user_id.clone()),
        is_published: Set(post_form.is_published.clone()),
        ..Default::default()
    }
    .save(conn.as_ref())
    .await
    .expect("could not insert post");

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

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all);
    cfg.service(get_by_id);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}