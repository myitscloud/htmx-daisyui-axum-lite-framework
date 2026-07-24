# Agents.md

## Run

```
cargo run                     # start server on http://localhost:3000
cargo watch -c -x run         # dev mode (requires `cargo install cargo-watch`)
```

## Architecture

- **SPA over HTMX**: Axum returns full-page HTML on direct navigation or HTML fragments on HTMX (HX-Request header) requests. The `response_middleware` in `src/response.rs` handles this: HTMX partials are rendered raw; full-page navigations wrap the partial inside `templates/index.html` via `{{ content | safe }}`.
- **State**: `Arc<AppState>` holds a Tera engine, in-memory data (100 users, 50 products, 20 blog posts via `fake` crate), and a cached GitHub API response. No persistence layer.
- **Routes** (defined in `src/routes.rs`): `/` (dashboard stats), `/users` (paginated list), `/users/:id/view` (detail), `/products`, `/products/:id/view`, `/blog`, `/blog/:id`, `/repos` (live GitHub search), `/about`, fallback to 404.
- **Pagination** (`src/pagination.rs`): 10 items/page, page links centered around current page.
- **Live data**: `/repos` fetches from GitHub search API (unauthenticated, 60 req/hr limit). Results cached in-memory for 5 minutes.

## Testing

No tests or CI exist. No lint, formatter, or type-check configs.

## Templates

Located in `templates/`: `index.html` (DaisyUI shell), `dashboard.html`, `about.html`, `404.html`, `user/list.html`, `user/view.html`, `products/list.html`, `products/view.html`, `blog/list.html`, `blog/view.html`, `github.html`. Tera (Jinja2-style syntax). Uses DaisyUI 4 + Tailwind CSS 3 (CDN).
