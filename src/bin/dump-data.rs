use mediumrare::client::{Client, PostDataClient};

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let c = Client;
    let data = c.get_post_data(&input).unwrap();

    println!("{}", serde_json::to_string(&data).unwrap());
}
