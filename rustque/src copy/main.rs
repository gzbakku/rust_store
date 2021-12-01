
mod io;
mod config;
mod que;
mod map;
mod disk;
mod workers;

pub use config::Config;
pub use que::Que;
// use tokio;

use std::time::Instant;

#[tokio::main]
async fn main() {

    let hold = Instant::now();

    if false {
        io::read_full(
            "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string()
        ).await;
    }

    let mut que:Que;
    match Que::new(Config::new(
        "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
        10000,
        10
    )).await{
        Ok(v)=>{
            // println!("que made successfully");
            que = v;
        },
        Err(e)=>{
            println!("!!! failed-que::new => {:?}",e);
            return;
        }
    }

    println!("que initiated : {:?}",hold.elapsed());

    if false{que.print_map().await;}

    for i in 0..5000{
        match que.add(vec![1,2,3]).await{
            Ok(_)=>{
                println!(">>> success-que-add {:?}",i);
            },
            Err(_)=>{
                println!("!!! failed-que-add");
            }
        }
    }

    println!("write complete : {:?}",hold.elapsed());

    if true{que.print_map().await;}

    if false {
        io::read_full(
            "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string()
        ).await;
    }

}

//que(message)(await confirm)->map(message)->disk(message)(submit confirm)

