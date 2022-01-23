

//api
use rustque::{Que,Config};

#[tokio::main]
async fn main(){

    //---------------------------
    //initiate que
    //---------------------------
    let mut que:Que;
    match Que::new(Config::new(
        vec![
            "D://workstation/expo/rust/rust_store/test/rustque/que1.rustque".to_string(),
            "D://workstation/expo/rust/rust_store/test/rustque/que2.rustque".to_string(),
            "D://workstation/expo/rust/rust_store/test/rustque/que3.rustque".to_string()
        ],                  //que files
        500000000,          //min que size on disk 
        5000000,            //expand file on disk by this many bytes when full
        100                 //no of disk workers per que file
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
    //write items to the que
    //---------------------------
    if true {
        match que.add(vec![1,2,3]).await{
            Ok(mut que_response)=>{
                collect.push(async move{
                    que_response.check().await
                });
            },
            Err(_e)=>{
                println!("!!! failed-que-add : {:?}",_e);
            }
        }
    }

    //---------------------------
    // please enable get, remove and reset
    // functions once at a time or write 
    // que items for each of them
    //---------------------------

    //---------------------------
    //get qued item from que
    //---------------------------
    if true{
        match que.next().await{
            Ok(mut next_response)=>{
                let _quer_resp = next_response.check().await;
                if !_quer_resp {break;}
                match next_response.data().await{
                    Some((value,pointer))=>{
                        println!("value : {:?}",value);
                    },
                    None=>{}
                }
            },
            Err(_e)=>{
                println!("!!! failed-que-get : {:?}",_e);
            }
        }
    }

    //---------------------------
    //remove item from que
    //---------------------------
    if true{
        match que.next().await{
            Ok(mut next_response)=>{
                if next_response.check().await {
                    match next_response.data().await{
                        Some((_value,pointer))=>{
                            match que.remove(pointer).await{
                                Ok(mut remove_response)=>{
                                    let remove_resp = remove_response.check().await;
                                    println!("remove resp : {:?}",remove_resp);
                                },
                                Err(_e)=>{
                                    println!("!!! failed-que-remove : {:?}",_e);
                                }
                            }
                        },
                        None=>{}
                    }
                }
            },
            Err(_e)=>{
                println!("!!! failed-que-get : {:?}",_e);
            }
        }
    }

    //---------------------------
    //reset item in que
    //---------------------------
    if true{
        match que.next().await{
            Ok(mut next_response)=>{
                if next_response.check().await {
                    match next_response.data().await{
                        Some((_value,pointer))=>{
                            match que.reset(pointer).await{
                                Ok(mut reset_response)=>{
                                    let reset_resp = reset_response.check().await;
                                    println!("reset resp : {:?}",reset_resp);
                                },
                                Err(_e)=>{
                                    println!("!!! failed-que-reset : {:?}",_e);
                                }
                            }
                        },
                        None=>{}
                    }
                }
            },
            Err(_e)=>{
                println!("!!! failed-que-get : {:?}",_e);
            }
        }
    }

}


//benchmark
use rustque::bechmark::{BenchmarkBuilder,Benchmark};

#[tokio::main]
async fn main(){

    //---------------------------
    //init benchmark builder
    //---------------------------
    let mut build = BenchmarkBuilder::new(
        "D://workstation/expo/rust/rust_store/test/rustque/bechmark_8.txt".to_string()
    );

    //---------------------------
    //add a benchmark
    //---------------------------
    build.add(Benchmark{
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

    //---------------------------
    //run the benchmarks
    //---------------------------
    build.run().await;

}