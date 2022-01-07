
mod io;
mod config;
mod que;
mod map;
mod disk;
mod workers;
mod locator;
mod response;
mod benchmark;

pub use config::Config;
pub use que::Que;

use tokio::sync::Notify;
use std::sync::Arc;
use std::time::Instant;

use futures::future::join_all;

#[tokio::main]
async fn main() {

    let cmd_line = std::env::args();
    for cmd in cmd_line{
        if cmd.contains("--simple"){
            simple().await;
            return;
        } else
        if cmd.contains("--bechmark"){
            bechmarks().await;
            return;
        }
    }
    
    bechmarks().await;

}

async fn bechmarks(){

    let mut build = benchmark::BenchmarkBuilder::new(
        "D://workstation/expo/rust/rust_store/test/rustque/bechmark_8.txt".to_string()
    );

    //5000

    if false {
        build.add(benchmark::Benchmark{
            no_of_writers:10,
            no_of_writes:5000,
            map_files:vec![
                "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string(),
            ],
            write_size:256,
            min_que_size:10000000,
            expansion_size:5000000,
            no_of_disk_workers:10
        });
    }

    //50,000

    if false {
        build.add(benchmark::Benchmark{
            no_of_writers:10,
            no_of_writes:50000,
            map_files:vec![
                "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string(),
            ],
            write_size:256,
            min_que_size:50000000,
            expansion_size:25000000,
            no_of_disk_workers:10
        });
    }

    //100,000

    if true {
        build.add(benchmark::Benchmark{
            no_of_writers:10,
            no_of_writes:100000,
            map_files:vec![
                "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string()
            ],
            write_size:256,
            min_que_size:100000000,
            expansion_size:50000000,
            no_of_disk_workers:10
        });
        build.add(benchmark::Benchmark{
            no_of_writers:10,
            no_of_writes:100000,
            map_files:vec![
                "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string()
            ],
            write_size:512,
            min_que_size:100000000,
            expansion_size:50000000,
            no_of_disk_workers:10
        });
        build.add(benchmark::Benchmark{
            no_of_writers:10,
            no_of_writes:200000,
            map_files:vec![
                "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string()
            ],
            write_size:256,
            min_que_size:100000000,
            expansion_size:50000000,
            no_of_disk_workers:10
        });
    }

    if false {
        build.add(benchmark::Benchmark{
            no_of_writers:10,
            no_of_writes:100000,
            map_files:vec![
                "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que4.rustque".to_string(),
                "D://workstation/expo/rust/rust_store/test/rustque/que5.rustque".to_string()
            ],
            write_size:256,
            min_que_size:100000000,
            expansion_size:50000000,
            no_of_disk_workers:10
        });
    }


    build.run().await;

}

async fn simple(){

    let hold = Instant::now();

    //---------------------------
    //initiate que
    //---------------------------
    let mut que:Que;
    match Que::new(Config::new(
        vec![
            "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
            "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
            "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string(),
            // "D://workstation/expo/rust/rust_store/test/rustque/que4.rustque".to_string(),
        ],
        500000000,
        5000000,
        100
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

    let mut big_value = vec![];
    let mut last_put = 0;
    loop {
        if big_value.len() == 512{break;}
        if last_put == 200{last_put = 1;} else {last_put += 1;}
        big_value.push(last_put);
    }

    // println!("{:?}",big_value);
    //1_000_000

    //---------------------------
    //write items to the que
    //---------------------------
    if false{
        for _ in 0..1{
            let write_time_final = Instant::now();
            let sleeper = Arc::new(Notify::new());
            let waker = sleeper.clone();
            let no_of_spawns = 50;
            for _nsp in 0..no_of_spawns{
                let que_to_move = que.clone();
                let waker_to_move = waker.clone();
                let hold_big_value = big_value.clone();
                tokio::spawn(async move {
                    let write_spawn_time = Instant::now();
                    let mut que = que_to_move;
                    let mut collect = Vec::new();
                    for _n in 0..10000{
                        match que.add(hold_big_value.clone()).await{
                            Ok(mut que_response)=>{
                                collect.push(async move{
                                    que_response.check().await
                                });
                                // let _quer_resp = que_response.check().await;
                                // println!("{:?} add resp : {:?}",_n,_quer_resp);
                                // println!(">>> success-que-add {:?}",_n);
                            },
                            Err(_e)=>{
                                println!("!!! failed-que-add : {:?}",_e);
                            }
                        }
                    }
                    let mut failed = 0;
                    let mut success = 0;
                    for r in join_all(collect).await.iter(){
                        if !r{failed += 1;} else {success += 1;}
                    }
                    println!("{:?} write_spawn_time : {:?} {:?} {:?}",_nsp,write_spawn_time.elapsed(),failed,success);
                    waker_to_move.notify_one();
                });
            }
            for _ in 0..no_of_spawns{
                sleeper.notified().await
            }
            println!("write_time_final : {:?}",write_time_final.elapsed());
        }
    }

    //---------------------------
    //get and remove items from que
    //---------------------------
    if false{
        let remove_time_final = Instant::now();
        loop{
            match que.next().await{
                Ok(mut next_response)=>{
                    let _quer_resp = next_response.check().await;
                    if !_quer_resp {break;}
                    // println!("next resp : {:?}",_quer_resp);

                    match next_response.data().await{
                        Some((_value,pointer))=>{

                            // println!("pointer : {:?}",pointer);

                            if true{
                                match que.remove(pointer).await{
                                    Ok(mut remove_response)=>{
                                        let remove_resp = remove_response.check().await;
                                        println!("remove resp : {:?}",remove_resp);
                                    },
                                    Err(_e)=>{
                                        println!("!!! failed-que-remove : {:?}",_e);
                                    }
                                }
                            }

                            if false{
                                match que.reset(pointer).await{
                                    Ok(mut reset_response)=>{
                                        let reset_resp = reset_response.check().await;
                                        println!("reset resp : {:?}",reset_resp);
                                    },
                                    Err(_e)=>{
                                        println!("!!! failed-que-reset : {:?}",_e);
                                    }
                                }
                            }

                        },
                        None=>{}
                    }
                    // break;
                },
                Err(_e)=>{
                    println!("!!! failed-que-get : {:?}",_e);
                    break;
                }
            }
        }
        println!("remove_time_final : {:?}",remove_time_final.elapsed());
    }

    // if false{
    //     for _ in 0..5{
    //         match que.get().await{
    //             Ok(_v)=>{
    //                 println!(">>> success-que-get {:?}",_v);
    //                 match que.reset(_v.1).await{
    //                     Ok(_v)=>{
    //                         println!(">>> success-que-reset");
    //                     },
    //                     Err(_e)=>{
    //                         println!("!!! failed-que-reset : {:?}",_e);
    //                     }
    //                 }
    //             },
    //             Err(_e)=>{
    //                 println!("!!! failed-que-get : {:?}",_e);
    //             }
    //         }
    //     }
    // }

    println!("final in : {:?}",hold.elapsed());

}

//que(message)(await confirm)->map(message)->disk(message)(submit confirm)

