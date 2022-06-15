use actix_web::HttpResponse;
use serde_json::json;

pub fn resp_log_reg_status(data: &'static str) -> HttpResponse{
    let data = json!({
        "response_type": "log_reg_status",
        "message": data
    });
    HttpResponse::Ok().body(data.to_string())
}

pub fn resp_give_uuid(uuid: String) -> HttpResponse{
    let data = json!({
        "response_type": "uuid",
        "message": uuid
    });
    HttpResponse::Ok().body(data.to_string())
}
