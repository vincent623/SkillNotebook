use crate::domain::common::AppResponse;

pub fn not_found<T>(entity: &str, identifier: &str) -> AppResponse<T> {
    AppResponse::failure("not_found", format!("{} not found: {}", entity, identifier))
}

pub fn not_implemented<T>(feature: &str) -> AppResponse<T> {
    AppResponse::failure(
        "not_implemented",
        format!("{} is scaffolded but not wired yet", feature),
    )
}
