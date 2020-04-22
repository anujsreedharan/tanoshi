use crate::auth::auth::Auth;
use crate::auth::Claims;
use crate::filters::{with_authorization, with_db};
use crate::handlers::auth as auth_handler;
use crate::handlers::history as history_handler;
use crate::handlers::history::{History, HistoryParam};
use postgres::Client;
use std::sync::{Arc, Mutex};
use warp::Filter;

pub fn history(
    secret: String,
    db: Arc<Mutex<Client>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_history(secret.clone(), db.clone()).or(add_history(secret.clone(), db))
}

fn get_history(
    secret: String,
    db: Arc<Mutex<Client>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "history")
        .and(warp::get())
        .and(with_authorization(secret))
        .and(warp::query::<HistoryParam>())
        .and(with_db(db))
        .and_then(history_handler::get_history)
}

fn add_history(
    secret: String,
    db: Arc<Mutex<Client>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "history")
        .and(warp::post())
        .and(with_authorization(secret))
        .and(json_body())
        .and(with_db(db))
        .and_then(history_handler::add_history)
}

fn json_body() -> impl Filter<Extract = (History,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
