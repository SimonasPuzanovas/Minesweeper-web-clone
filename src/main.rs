use actix::{Actor, Addr};
use actix_web::{http::header::ContentType, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use bson::{doc};
use mongodb::{options::ClientOptions, Client};
mod minesweeper;
use service::Services;
mod service;
use requests::RegisterForm;
mod requests;
mod responses;
use myws::MyWs;
mod games;
mod myws;
use games::Games;
use std::time::{Instant};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("webapp");

    let service = Services::new(db.collection("users"));
    let game = Games::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .app_data(web::Data::new(game.clone()))
            .service(web::resource("/ws").route(web::get().to(ws_connect)))
            .service(web::resource("static/{src}").route(web::get().to(static_req)))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/register").route(web::post().to(register)))
            .service(web::resource("/login").route(web::post().to(login)))
            .service(web::resource("/gamelist.json").route(web::get().to(gamelist)))
            .service(web::resource("static/images/{img}").route(web::get().to(render_image)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn index(req: HttpRequest, data: web::Data<Services>) -> HttpResponse {
    data.handle_cookie(&req).await
}
async fn static_req(_req: HttpRequest, src: web::Path<String>) -> HttpResponse{
    if src.chars().last().unwrap() == 's'{
        HttpResponse::Ok()
            .content_type(ContentType(mime::TEXT_CSS))
            .body(std::fs::read(format!("static/{}", src)).unwrap())
    }
    else {HttpResponse::Ok().body(std::fs::read(format!("static/{}", src)).unwrap())}
}

async fn ws_connect(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Addr<Games>>,
    service: web::Data<Services>) -> HttpResponse {

    let bson = service.users.find_one(doc! {"uuid": req.cookie("uuid").unwrap().value()}, None)
        .await.unwrap().unwrap();
    
    let resp = ws::start(
        MyWs {
            heartbeat: Instant::now(),
            gameid: String::new(),
            addr: data.get_ref().clone(),
            username: bson.get_str("name")
                .unwrap().to_string(),
        },
        &req,
        stream,
    ).unwrap();
    resp
}

async fn register(post: web::Json<RegisterForm>, data: web::Data<Services>) -> HttpResponse {
    data.add_user(&post.name, &post.password).await
}

async fn login(post: web::Json<RegisterForm>, data: web::Data<Services>) -> HttpResponse {
    data.login_user(&post.name, &post.password).await
}

async fn gamelist(data: web::Data<Addr<Games>>) -> HttpResponse{
    match data.send(games::GameList{}).await{
        Ok(data) => HttpResponse::Ok().body(data.unwrap()),
        Err(_) => HttpResponse::NoContent().body("erorr"),        
    }
}

async fn render_image(img: web::Path<String>) -> HttpResponse {
    match std::fs::read(format!("static/images/{}", img)) {
        Ok(img) => HttpResponse::Ok().body(img),
        Err(_) => HttpResponse::NotFound().body(std::fs::read("static/images/error.jpeg").unwrap()),
    }
}
