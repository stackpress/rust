//! Finite stdio entry point; the HTTP adapter handles bytes without opening a listener.
use std::io::Read;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let output = api_checkpoint_web::finite::run(serde_json::from_str(&input)?).await?;
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
