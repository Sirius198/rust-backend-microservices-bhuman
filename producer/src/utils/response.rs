use axum::{body::Body, response::Response};

pub fn send_reponse(code: i32, body: Body) -> Result<Response<Body>, Response<Body>> {
    match code {
        200 => {
            Ok(Response::builder()
                .status(200)
                .body(body)
                .unwrap())
        }
        400 => {
            Err(Response::builder()
                .status(400)
                .body(body)
                .unwrap())
        }
        _ => {
            Err(Response::builder()
                .status(500)
                .body(body)
                .unwrap())
        }
    }
}