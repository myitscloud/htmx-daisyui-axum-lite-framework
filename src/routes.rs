use std::time::Instant;

use axum::{
    http::header,
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use serde_json::{json, Value};

use crate::{response::HxResponse, ExtAppState, GithubCache};

pub static CSS: &str = include_str!("../static/output.css");

pub fn routes() -> Router {
    Router::new()
        .route("/static/output.css", get(get_css))
        .route("/", get(index))
        .route("/about", get(about))
        .route("/repos", get(repos_search))
        .route("/repos/starred", get(repos_starred))
        .route("/repos/trending", get(repos_trending))
        .route("/repos/forked", get(repos_forked))
        .fallback(notfound)
}

async fn get_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], CSS)
}

async fn index() -> HxResponse {
    HxResponse::view("welcome")
}

async fn repos_search(Extension(state): Extension<ExtAppState>) -> HxResponse {
    let repos = fetch_or(
        &state.github_cache,
        |c| (&c.repos, &c.repos_fetched),
        |c, data, time| {
            c.repos = data;
            c.repos_fetched = time;
        },
        "https://api.github.com/search/repositories?q=awesome-llm&sort=stars&order=desc&per_page=12",
    )
    .await;

    HxResponse::new("github", json!({"repos": repos, "title": "Search: awesome-llm", "desc": "Top repos tagged awesome-llm from GitHub", "current_url": "/repos"}))
}

async fn repos_starred(Extension(state): Extension<ExtAppState>) -> HxResponse {
    let repos = fetch_or(
        &state.github_cache,
        |c| (&c.starred, &c.starred_fetched),
        |c, data, time| {
            c.starred = data;
            c.starred_fetched = time;
        },
        "https://api.github.com/search/repositories?q=stars:>1000&sort=stars&order=desc&per_page=12",
    )
    .await;

    HxResponse::new("github", json!({"repos": repos, "title": "Most Starred", "desc": "All-time top repositories by star count", "current_url": "/repos/starred"}))
}

async fn repos_trending(Extension(state): Extension<ExtAppState>) -> HxResponse {
    let repos = fetch_or(
        &state.github_cache,
        |c| (&c.trending, &c.trending_fetched),
        |c, data, time| {
            c.trending = data;
            c.trending_fetched = time;
        },
        "https://api.github.com/search/repositories?q=created:>2026-01-01&sort=stars&order=desc&per_page=12",
    )
    .await;

    HxResponse::new("github", json!({"repos": repos, "title": "Trending", "desc": "Repos created this year gaining stars fast", "current_url": "/repos/trending"}))
}

async fn repos_forked(Extension(state): Extension<ExtAppState>) -> HxResponse {
    let repos = fetch_or(
        &state.github_cache,
        |c| (&c.forked, &c.forked_fetched),
        |c, data, time| {
            c.forked = data;
            c.forked_fetched = time;
        },
        "https://api.github.com/search/repositories?q=forks:>1000&sort=forks&order=desc&per_page=12",
    )
    .await;

    HxResponse::new("github", json!({"repos": repos, "title": "Most Forked", "desc": "Top repositories by fork count", "current_url": "/repos/forked"}))
}

async fn fetch_or<F, G>(cache: &std::sync::Mutex<GithubCache>, get: F, set: G, url: &str) -> Vec<Value>
where
    F: Fn(&GithubCache) -> (&Vec<Value>, &Instant),
    G: Fn(&mut GithubCache, Vec<Value>, Instant),
{
    {
        let c = cache.lock().unwrap();
        let (data, fetched) = get(&c);
        if !data.is_empty() && fetched.elapsed().as_secs() < 300 {
            return data.clone();
        }
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "htmx-demo/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let data: Value = r.json().await.unwrap();
            let items = data["items"].as_array().cloned().unwrap_or_default();
            let now = Instant::now();
            let mut c = cache.lock().unwrap();
            set(&mut c, items.clone(), now);
            items
        }
        _ => {
            let c = cache.lock().unwrap();
            let (data, _) = get(&c);
            if !data.is_empty() {
                data.clone()
            } else {
                vec![]
            }
        }
    }
}

async fn about() -> HxResponse {
    HxResponse::view("about")
}

async fn notfound() -> HxResponse {
    HxResponse::view("404")
}
