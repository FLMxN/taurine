use dotenv;
fn main() {
    dotenv::dotenv().ok();
    println!("Building Taurine v{}", std::env::var("VERSION").unwrap_or_else(|_| "unknown".into()));
}