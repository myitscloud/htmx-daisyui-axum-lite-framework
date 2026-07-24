use std::{fs, sync::{Arc, Mutex}, time::Instant};

use axum::{middleware::from_fn, Extension};
use response::response_middleware;
use routes::routes;
use serde_json::Value;

mod response;
mod routes;

pub struct GithubCache {
    pub repos: Vec<Value>,
    pub repos_fetched: Instant,
    pub starred: Vec<Value>,
    pub starred_fetched: Instant,
    pub trending: Vec<Value>,
    pub trending_fetched: Instant,
    pub forked: Vec<Value>,
    pub forked_fetched: Instant,
}

pub struct AppState {
    pub h: tera::Tera,
    pub github_cache: Mutex<GithubCache>,
}

pub type ExtAppState = Arc<AppState>;

#[tokio::main]
async fn main() {
    let mut h = tera::Tera::new();

    for (name, path) in [
        ("index", "templates/index.html"),
        ("about", "templates/about.html"),
        ("404", "templates/404.html"),
        ("github", "templates/github.html"),
        ("welcome", "templates/welcome.html"),
    ] {
        h.add_raw_template(name, &fs::read_to_string(path).expect("Unable to read template"))
            .expect("Unable to register template");
    }

    let state = Arc::new(AppState {
        h,
        github_cache: Mutex::new(GithubCache {
            repos: vec![],
            repos_fetched: Instant::now(),
            starred: vec![],
            starred_fetched: Instant::now(),
            trending: vec![],
            trending_fetched: Instant::now(),
            forked: vec![],
            forked_fetched: Instant::now(),
        }),
    });

    let app = routes()
        .layer(from_fn(response_middleware))
        .layer(Extension(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening to: http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
