use crate::domain::common::AppResponse;

pub fn not_found<T>(entity: &str, identifier: &str) -> AppResponse<T> {
    AppResponse::failure("not_found", format!("{} not found: {}", entity, identifier))
}
