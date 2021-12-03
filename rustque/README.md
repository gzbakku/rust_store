# rustque

this is a fast and optimized tokio based persistant que for rust it writes Vec<u8> to a file on a disk, supported functions are add,get and remove, it keeps a data map in memory so large data sets should be supported with appropriarte memory.
   
## sample code  

```rust 

use rustque::{Que,Config};
use tokio::sync::Notify;
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() {

    let hold = Instant::now();

    //---------------------------
    //initiate que
    //---------------------------
    let mut que:Que;
    match Que::new(Config::new(
        "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
        5_000_000,
        5
    )).await{
        Ok(v)=>{
            que = v;
            println!("que initiated : {:?}",hold.elapsed());
        },
        Err(e)=>{
            println!("!!! failed-que::new => {:?}",e);
            return;
        }
    }

    //---------------------------
    //write ietsm to the que
    //---------------------------
    if true{
        let write_time_final = Instant::now();
        let sleeper = Arc::new(Notify::new());
        let waker = sleeper.clone();
        let no_of_spawns = 10;
        for _ in 0..no_of_spawns{
            let que_to_move = que.clone();
            let waker_to_move = waker.clone();
            tokio::spawn(async move {
                // let write_spawn_time = Instant::now();
                let mut que = que_to_move;
                for _n in 0..5000{
                    match que.add(vec![1,2,3]).await{
                        Ok(_)=>{
                            // println!(">>> success-que-add {:?}",_n);
                        },
                        Err(_e)=>{
                            println!("!!! failed-que-add : {:?}",_e);
                        }
                    }
                }
                // println!("write_spawn_time : {:?}",write_spawn_time.elapsed());
                waker_to_move.notify_one();
            });
        }
        for _ in 0..no_of_spawns{
            sleeper.notified().await
        }
        println!("write_time_final : {:?}",write_time_final.elapsed());
    }

    //---------------------------
    //get and remove items from que
    //---------------------------
    if false{
        let remove_time_final = Instant::now();
        loop{
            match que.get().await{
                Ok(_v)=>{
                    // println!(">>> success-que-get {:?}",_v);
                    match que.remove(_v.1).await{
                        Ok(_v)=>{
                            // println!(">>> success-que-remove {:?}",_v);
                        },
                        Err(_e)=>{
                            println!("!!! failed-que-remove : {:?}",_e);
                        }
                    }
                },
                Err(_e)=>{
                    println!("!!! failed-que-get : {:?}",_e);
                    break;
                }
            }
        }
        println!("remove_time_final : {:?}",remove_time_final.elapsed());
    }

    println!("final in : {:?}",hold.elapsed());

}

```


