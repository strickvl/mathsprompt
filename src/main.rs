use actix_web::{web, App, HttpResponse, HttpServer, middleware, Responder};
use actix_web::http::header;
use serde::Deserialize;
use tokio_postgres::NoTls;

#[derive(Deserialize)]
struct FormData {
    text: String,
    tags: String,
}

async fn insert_question(form: web::Json<FormData>) -> impl Responder {
    let (client, connection) = tokio_postgres::connect("postgresql://strickvl:alex@localhost:5432/mathsprompt", NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let tags: Vec<&str> = form.tags.split(',').collect();

    let row = client.query_one("INSERT INTO questions (text, next_due, autogenerated, created_at, updated_at) VALUES ($1, NOW(), false, NOW(), NOW()) RETURNING id", &[&form.text]).await.unwrap();
    let question_id: i32 = row.get(0);

    let mut tag_ids: Vec<i32> = Vec::new();
    for tag in tags {
        let rows = client.query("SELECT id FROM tags WHERE name = $1", &[&tag]).await.unwrap();
        if rows.is_empty() {
            let row = client.query_one("INSERT INTO tags (name, created_at, updated_at) VALUES ($1, NOW(), NOW()) RETURNING id", &[&tag]).await.unwrap();
            let tag_id: i32 = row.get(0);
            tag_ids.push(tag_id);
        } else {
            let tag_id: i32 = rows[0].get(0);
            tag_ids.push(tag_id);
        }
    }

    for tag_id in tag_ids {
        client.execute("INSERT INTO question_tag (question_id, tag_id, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())", &[&question_id, &tag_id]).await.unwrap();
    }

    HttpResponse::Ok().body("Question added successfully!")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(
                middleware::DefaultHeaders::new()
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "GET, POST")
                    .header("Access-Control-Allow-Headers", "Content-Type")
            )
            .route("/", web::post().to(insert_question))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
