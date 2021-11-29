use std::sync::Arc;
use crate::workers::Signal;
use crate::config::{MapConfig,MapMessage,DiskMessage};
use tokio::sync::{Notify,Mutex};

pub async fn init(config:MapConfig){

    let mut config = config;

    loop{

        let message:MapMessage;
        match config.receiver.recv_async().await{
            Ok(m)=>{
                message = m;
            },
            Err(_)=>{
                break;
            }
        }

        match message{
            MapMessage::Add(value)=>{
                handle_add(&mut config,value).await;
            }
        }

    }

}

async fn handle_add(config:&mut MapConfig,message:(Vec<u8>,Arc<Mutex<Signal>>,Arc<Notify>)){

    println!("handling add");

    //find the biggest value
    let biggest_value:u64;
    if config.items.len() == 0{
        biggest_value = 1;
    } else{
        biggest_value = config.items[config.items.len()-1]+1;
    }

    match config.reader.fill(biggest_value.to_be_bytes().to_vec(),message.0.len()){
        Ok(_)=>{
            println!("added to map");
            println!("{:?}",config.reader.map);
        },
        Err(e)=>{
            if e == "full"{
                println!("expand me");

                //expand
                let signal = Signal::new();
                let waker = Arc::new(Notify::new());
                let sleeper = waker.clone();
                match config.disk_sender.send_async(DiskMessage::Expand((signal.clone(),waker))).await{
                    Ok(_)=>{},
                    Err(_)=>{
                        message.2.notify_waiters();
                    }
                }
                sleeper.notified().await;
                if !Signal::check(signal).await{
                    message.2.notify_waiters();
                }

                //try again
                println!("expand complete");

            } else {
                message.2.notify_waiters();
            }
        }
    }

}

