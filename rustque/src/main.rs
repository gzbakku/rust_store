
mod io;
mod config;
mod que;
mod map;
mod disk;
mod workers;

pub use config::Config;
pub use que::Que;
// use tokio;

#[tokio::main]
async fn main() {

    let mut que:Que;
    match Que::new(Config::new(
        "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
        10,
        5
    )).await{
        Ok(v)=>{
            println!("que made successfully");
            que = v;
        },
        Err(e)=>{
            println!("!!! failed-que::new => {:?}",e);
            return;
        }
    }

    match que.add(vec![1,2,3]).await{
        Ok(_)=>{
            println!(">>> success-que-add");
        },
        Err(_)=>{
            println!("!!! failed-que-add");
        }
    }

}

//que(message)(await confirm)->map(message)->disk(message)(submit confirm)

