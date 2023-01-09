use mediumrare::client::QueryResponse;
use mediumrare::content::Render;

fn main() {
    let input = std::env::args().nth(1).unwrap();
    let file = std::fs::read(input).unwrap();
    let data: QueryResponse = serde_json::from_slice(&file).unwrap();

    println!("{}", data.get_post().render().unwrap().to_string());
}
