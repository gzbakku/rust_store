use crate::workers::{Signal,SignalData,Pointer};
// use crate::workers::{debug_error,debug_message};
// use crate::workers::Debugger;
use tokio::sync::{Mutex};
use crate::config::{DiskConfig,DiskMessage,DiskAddMessage,DiskGetMessage,DiskRemoveMessage};
use std::sync::Arc;
use tokio::fs::{File,OpenOptions};
use crate::io::{expand,write_chunk,read_chunk,remove_chunk};
// use std::time::Instant;
// use crate::workers::debug_error;

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
                handle_add(value,&mut file).await;
            },
            DiskMessage::Get(value)=>{
                handle_get(value,&mut file).await;
            },
            DiskMessage::Remove(value)=>{
                handle_remove(value,&mut file).await;
            }
        }

    }

}

async fn handle_remove(message:DiskRemoveMessage,file:&mut File){
    match remove_chunk(file, message.boundry.0, message.boundry.1 - message.boundry.0 + 1).await{
        Ok(_)=>{
            Signal::ok(message.signal).await;
        },
        Err(_)=>{
            Signal::error(message.signal).await;
        }
    }
}

async fn handle_get(message:DiskGetMessage,file:&mut File){
    let len = message.value_boundry.1 - message.value_boundry.0 + 1;
    let mut read_buffer:Vec<u8> = Vec::with_capacity(len);
    match read_chunk(file, &mut read_buffer, message.value_boundry.0 as u64, len as u64).await{
        Ok(_)=>{
            Signal::data(message.signal,SignalData::Value((read_buffer,Pointer{
                item_index:message.item_index,
                map_index:message.map_index
            }))).await;
        },
        Err(_)=>{
            Signal::error(message.signal).await;
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
            // debug_message("write_complete-add-disk", DEBUG);
            Signal::ok(message.signal).await;
            // println!("signal marked");
            //Debuggerupdate(&message.debugger, "added to disk").await;
            // debug_message("added to disk",DEBUG);
        },
        Err(_)=>{
            // debug_error("failed-add-disk",ERROR);
            Signal::error(message.signal).await;
            //Debuggererror(&message.debugger, "failed-handle_add-disk.rs").await;
            // debug_message("failed add to disk",DEBUG);
            // debug_error("failed-handle_add-disk.rs",ERROR);
        }
    }
    //Debuggerupdate(&message.debugger, "handle_add notified").await;
    // debug_message("handle_add notified",DEBUG);
}

//(Arc<Mutex<Signal>>,Arc<Notify>),file:&mut File
async fn handle_expand(config:&mut DiskConfig,message:Arc<Mutex<Signal>>,file:&mut File){
    match expand(file,&config.frame_size).await{
        Ok(_)=>{
            Signal::ok(message).await;
        },
        Err(_)=>{
            Signal::error(message).await;
            // debug_error("failed-handle_expand-disk.rs",ERROR);
        }
    }
}