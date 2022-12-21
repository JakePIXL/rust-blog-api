use actix_web::web;

mod posts;
mod users;

use posts::init_routes as init_posts_routes;
use users::init_routes as init_users_routes;


pub fn init_routes(cfg: &mut web::ServiceConfig) {
    init_posts_routes(cfg);
    init_users_routes(cfg);
}