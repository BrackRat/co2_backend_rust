use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use serde_json;
use chrono::{DateTime};

#[allow(warnings, unused)]
mod db;

use db::*;
use prisma_client_rust::Direction;


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateReqBody {
    timestamp: i64,
    co2: i32,
    tvoc: i32,
}

#[post("/update")]
async fn update(client: web::Data<PrismaClient>, body: web::Json<UpdateReqBody>) -> impl Responder {
    let update = client
        .history()
        .create(
            body.timestamp,
            body.co2,
            body.tvoc,
            vec![],
        )
        .exec()
        .await
        .unwrap();
    HttpResponse::Ok().json(update)
}

#[get("/status")]
async fn get_latest(client: web::Data<PrismaClient>) -> impl Responder {
    // ToDo Performance improvement
    let latest = client
        .history()
        .find_many(vec![])
        .order_by(history::timestamp::order(Direction::Desc))
        .exec()
        .await
        .unwrap()[0].clone();

    // generate formatted time from latest.timestamp like "%Y-%m-%d %H:%M:%S"
    let dt = DateTime::from_timestamp(latest.timestamp, 0);
    let formatted_time = dt.expect("Format time failed").format("%Y-%m-%d %H:%M:%S").to_string();

    let response = serde_json::json!({
        "code": 200,
        "data": {
            "co2": latest.co_2,
            "tvoc": latest.tvoc,
            "time": formatted_time
        }
    });

    HttpResponse::Ok().json(response)
}


#[get("/history")]
async fn get_history_chart(client: web::Data<PrismaClient>) -> impl Responder {
    let latest = client
        .history()
        .find_many(vec![])
        .order_by(history::timestamp::order(Direction::Desc))
        .exec()
        .await
        .unwrap();

    let history_data = latest
        .iter()
        .map(|history| {
            let dt = DateTime::from_timestamp(history.timestamp, 0);
            let formatted_time = dt.expect("F").format("%Y-%m-%d %H:%M:%S").to_string();
            (formatted_time, history.co_2, history.tvoc)
        })
        .collect::<Vec<(String, i32, i32)>>();

    let response = serde_json::json!({
        "code": 200,
        "data": {
            "history_data": history_data
        }
    });

    HttpResponse::Ok().json(response)
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
            .service(get_history_chart)
    })
        .bind(("127.0.0.1", 5010))?
        .run()
        .await
}