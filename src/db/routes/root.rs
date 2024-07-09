use crate::db::utils::request as req;
use crate::db::utils::response as res;
use std::collections::HashMap;

pub fn handler(request: &req::Request) -> res::Response<String> {
    match request.method.as_str() {
        "get" => get(),
        _ => res::get_not_allowed_response(),
    }
}

fn get() -> res::Response<String> {
    let mut map = HashMap::new();
    map.insert("status", "ok");
    res::create_response(200, Some(map))
}
