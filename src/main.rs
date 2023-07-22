use warp::Filter;

#[tokio::main]
async fn main() {
    let hello_world = warp::path("hello").map(|| warp::reply::html("<h1>Hello, world!</h1>"));

    warp::serve(hello_world).run(([127, 0, 0, 1], 3030)).await;
}
