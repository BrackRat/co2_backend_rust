use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};

#[allow(warnings, unused)]
mod db;
use db::*;
use prisma_client_rust::Direction;


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug,Serialize,Deserialize)]
struct UpdateReqBody {
    timestamp: u64,
    co2: i32,
    tvoc: i32,
}

#[post("/update")]
async fn update(client: web::Data<PrismaClient>, body: web::Json<UpdateReqBody>) -> impl Responder {
    let update = client
        .history()
        .create(
            body.timestamp as i32,
            body.co2,
            body.tvoc,
            vec![]
        )
        .exec()
        .await
        .unwrap();
    HttpResponse::Ok().json(update)
}

#[get("/status")]
async fn get_latest(client: web::Data<PrismaClient>) -> impl Responder {
    let latest = client
        .history()
        .find_many(vec![])
        .order_by(history::timestamp::order(Direction::Desc))
        .exec()
        .await
        .unwrap()[0].clone();
    HttpResponse::Ok().json(latest)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = web::Data::new(PrismaClient::_builder().build().await.unwrap());

    println!("Server Start");

    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .service(hello)
            .service(update)
            .service(get_latest)
    })
        .bind(("127.0.0.1", 5010))?
        .run()
        .await
}