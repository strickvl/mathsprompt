use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use chrono::{Duration, Utc};

use regex::Regex;
use serde::Deserialize;
use serde_json::json;
use tokio_postgres::NoTls;

#[macro_use]
extern crate log;

#[derive(Deserialize)]
struct FormData {
    text: String,
    tags: String,
}

async fn insert_question_and_variants(form: web::Json<FormData>) -> impl Responder {
    let database_url =
        std::env::var("MATHSPROMPT_DATABASE_URL").expect("MATHSPROMPT_DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let tags: Vec<&str> = form.tags.split(',').collect();

    let mut tag_ids: Vec<i32> = Vec::new();
    for tag in tags {
        let rows = client
            .query("SELECT id FROM tags WHERE name = $1", &[&tag])
            .await
            .unwrap();
        if rows.is_empty() {
            let row = client.query_one("INSERT INTO tags (name, created_at, updated_at) VALUES ($1, NOW(), NOW()) RETURNING id", &[&tag]).await.unwrap();
            let tag_id: i32 = row.get(0);
            tag_ids.push(tag_id);
        } else {
            let tag_id: i32 = rows[0].get(0);
            tag_ids.push(tag_id);
        }
    }

    // New code to generate variants
    let openai_client = Client::new();
    let prompt = format!("I have this question:\n\n\"\"\"\n{}\n\"\"\"\n\nAnd I need you to generate 5 new questions that are similar in form, but with different values / variables (as is appropriate). The first should be the question above but reformatted and reworded if necessary to make it clear.\n\nThen give me the 5 generated questions. (So there should be 6 questions at the end). Each question should be separated by 7 @ symbols like: @@@@@@@ so that the returned text can be parsed. IMPORTANT: Just give me the question variations and the delimiter. Don't make any other comments before or afterwards in your response.", form.text);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512_u16)
        .model("gpt-4")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are a helpful assistant with experience as a Maths teacher for over 10 years.")
                .build().unwrap(),
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(&prompt)
                .build().unwrap(),
        ])
        .build().unwrap();

    let response = openai_client.chat().create(request).await.unwrap();
    let mut questions_added = 0;

    for choice in &response.choices {
        // Check if content is Some(String)
        if let Some(content) = &choice.message.content {
            // Step 1: Split the content into individual questions
            let re = Regex::new(r"@@@@@@@").unwrap();
            let questions: Vec<&str> = re.split(content).collect();

            for question in &questions {
                // Print out original question for debugging
                // println!("Original question: '{}'", question);

                // Step 2: Clean up the question text by removing numbering and leading whitespace
                let re_numbering = Regex::new(r"^\s*\d+[\.\)]?\s*").unwrap();
                let question_text = re_numbering.replace_all(question, "").trim().to_string();

                // Print out question after regex for debugging
                // println!("After regex: '{}'", question_text);

                if !question_text.trim().is_empty() {
                    let autogenerated = question_text != questions[0].to_string();

                    let row = client.query_one("INSERT INTO questions (text, next_due, autogenerated, created_at, updated_at) VALUES ($1, NOW(), $2, NOW(), NOW()) RETURNING id", &[&question_text, &autogenerated]).await.unwrap();
                    let new_question_id: i32 = row.get(0);

                    for tag_id in &tag_ids {
                        client.execute("INSERT INTO question_tag (question_id, tag_id, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())", &[&new_question_id, tag_id]).await.unwrap();
                    }

                    questions_added += 1;
                }
            }
        }
    }

    info!(
        "Added {} questions and {} tags to the database",
        questions_added,
        tag_ids.len()
    );

    HttpResponse::Ok().body("Question and variants added successfully!")
}

async fn get_random_question() -> impl Responder {
    let database_url =
        std::env::var("MATHSPROMPT_DATABASE_URL").expect("MATHSPROMPT_DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    match client
        .query_opt(
            "SELECT q.id, q.text FROM questions q
            INNER JOIN (
                SELECT question_id, AVG(ease) as avg_ease FROM question_answers
                GROUP BY question_id
            ) qa ON qa.question_id = q.id
            WHERE q.next_due <= NOW() AND qa.avg_ease <= 1
            ORDER BY RANDOM()
            LIMIT 1",
            &[],
        )
        .await
    {
        Ok(Some(row)) => {
            let question_text: String = row.get("text");
            let question_id: i32 = row.get("id"); // get the question ID
            println!("Sending question: {}", question_text);
            HttpResponse::Ok().json(json!({ "id": question_id, "text": question_text }))
            // return a JSON object
        }
        Ok(None) => {
            // No suitable question found, try to get a random question regardless of ease scores
            match client
                .query_one(
                    "SELECT id, text FROM questions ORDER BY RANDOM() LIMIT 1",
                    &[],
                )
                .await
            {
                Ok(row) => {
                    let question_text: String = row.get("text");
                    let question_id: i32 = row.get("id"); // get the question ID
                    println!("Sending random question: {}", question_text);
                    HttpResponse::Ok().json(json!({ "id": question_id, "text": question_text }))
                    // return a JSON object
                }
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    HttpResponse::InternalServerError().finish() // return a 500 error
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish() // return a 500 error
        }
    }
}

async fn submit_answer(answer: web::Json<Answer>) -> impl Responder {
    let database_url =
        std::env::var("MATHSPROMPT_DATABASE_URL").expect("MATHSPROMPT_DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Insert the new answer into the question_answers table
    client.execute(
        "INSERT INTO question_answers (question_id, answered_correctly, ease, answer_date, created_at) VALUES ($1, $2, $3, NOW(), NOW())",
        &[&answer.question_id, &answer.answered_correctly, &answer.ease],
    ).await.unwrap();

    // Update the question's next_due and updated_at based on ease
    let next_due = match answer.ease {
        0..=1 => Utc::now() + Duration::days(1), // if ease is 0 or 1, set next_due to tomorrow
        _ => Utc::now() + Duration::days(3),     // otherwise, set next_due to 3 days from now
    };

    client
        .execute(
            "
        UPDATE questions
        SET next_due = $1, updated_at = NOW()
        WHERE id = $2
    ",
            &[&next_due, &answer.question_id],
        )
        .await
        .unwrap();

    HttpResponse::Ok().json(json!({ "status": "success" }))
}

async fn get_all_tags() -> impl Responder {
    let database_url =
        std::env::var("MATHSPROMPT_DATABASE_URL").expect("MATHSPROMPT_DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client.query("SELECT * FROM tags", &[]).await.unwrap();

    let tags: Vec<String> = rows.iter().map(|row| row.get("name")).collect();

    HttpResponse::Ok().json(tags)
}

async fn get_random_question_by_tag(tag_name: web::Path<String>) -> impl Responder {
    let database_url =
        std::env::var("MATHSPROMPT_DATABASE_URL").expect("MATHSPROMPT_DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let tag_name_str = tag_name.into_inner();

    let rows = client
        .query(
            "SELECT q.id, q.text FROM questions q
        INNER JOIN question_tag qt ON qt.question_id = q.id
        INNER JOIN tags t ON t.id = qt.tag_id
        WHERE t.name = $1
        ORDER BY RANDOM()
        LIMIT 1",
            &[&tag_name_str],
        )
        .await
        .unwrap();

    if !rows.is_empty() {
        let question_text: String = rows[0].get("text");
        let question_id: i32 = rows[0].get("id");

        HttpResponse::Ok().json(json!({ "id": question_id, "text": question_text }))
    } else {
        HttpResponse::Ok().json(json!({ "status": "no questions found for this tag" }))
    }
}

#[derive(serde::Deserialize)]
struct Answer {
    question_id: i32,
    answered_correctly: bool,
    ease: i32,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(move || {
        App::new()
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", "*"))
                    .add(("Access-Control-Allow-Methods", "GET, POST"))
                    .add(("Access-Control-Allow-Headers", "Content-Type")),
            )
            .route("/", web::post().to(insert_question_and_variants))
            .route("/random", web::get().to(get_random_question))
            .route("/random/{tag}", web::get().to(get_random_question_by_tag)) // new route
            .route("/tags", web::get().to(get_all_tags)) // new route
            .route("/submit_answer", web::post().to(submit_answer))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
