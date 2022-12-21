use actix_web::web;

mod posts;
mod users;

use posts::{
    get_all,
    get_by_id,
    create,
    update,
    delete
};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all);
    cfg.service(get_by_id);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}