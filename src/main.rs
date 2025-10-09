use std::error::Error;
use crate::something::do_something;

mod something;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    do_something().await?;
    Ok(())
}
