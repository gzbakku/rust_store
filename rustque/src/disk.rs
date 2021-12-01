use crate::workers::Signal;
// use crate::workers::{debug_error,debug_message};
// use crate::workers::Debugger;
use tokio::sync::{Notify,Mutex};
use crate::config::{DiskConfig,DiskMessage,DiskAddMessage};
use std::sync::Arc;
use tokio::fs::{File,OpenOptions};
use crate::io::{expand,write_chunk};
// use std::time::Instant;

// const ERROR:bool = true;
// const DEBUG:bool = false;

pub async fn init(config:DiskConfig){

    let mut config = config;

    let mut file_builder = OpenOptions::new();
    file_builder.write(true)
    .read(true)
    .create(true);
    let mut file:File;
    match file_builder.open(&config.path).await{
        Ok(f)=>{
            file = f;
        },
        Err(_)=>{
            // debug_error("failed-open_file-disk.rs",ERROR);
            return;
        }
    }

    loop{

        let message:DiskMessage;
        match config.receiver.recv_async().await{
            Ok(m)=>{
                message = m;
            },
            Err(_)=>{
                // debug_error("failed-receive_message-disk.rs",ERROR);
                break;
            }
        }

        match message{
            DiskMessage::Expand(value)=>{
                handle_expand(&mut config,value,&mut file).await;
            },
            DiskMessage::Add(value)=>{
                // let hold = Instant::now();
                handle_add(value,&mut file).await;
                // println!("added in : {:?}",hold.elapsed());
            }
        }

    }

}


//(u64,Vec<u8>,Arc<Mutex<Signal>>,Arc<Notify>)
async fn handle_add(message:DiskAddMessage,file:&mut File){
    //Debuggerupdate(&message.debugger, "disk add message received").await;
    // debug_message("disk add message received",DEBUG);
    match write_chunk(file, message.start_at, message.value).await{
        Ok(_)=>{
            // println!("editing signal");
            Signal::ok(message.signal).await;
            // println!("signal marked");
            //Debuggerupdate(&message.debugger, "added to disk").await;
            // debug_message("added to disk",DEBUG);
        },
        Err(_)=>{
            //Debuggererror(&message.debugger, "failed-handle_add-disk.rs").await;
            // debug_message("failed add to disk",DEBUG);
            // debug_error("failed-handle_add-disk.rs",ERROR);
        }
    }
    message.notify.notify_one();
    //Debuggerupdate(&message.debugger, "handle_add notified").await;
    // debug_message("handle_add notified",DEBUG);
}

//(Arc<Mutex<Signal>>,Arc<Notify>),file:&mut File
async fn handle_expand(config:&mut DiskConfig,message:(Arc<Mutex<Signal>>,Arc<Notify>),file:&mut File){
    match expand(file,&config.frame_size).await{
        Ok(_)=>{
            Signal::ok(message.0).await;
        },
        Err(_)=>{
            // debug_error("failed-handle_expand-disk.rs",ERROR);
        }
    }
    message.1.notify_one();
}