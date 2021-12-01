use crate::workers::{Signal,debug_error,debug_message};
use tokio::sync::{Notify,Mutex};
use crate::config::{DiskConfig,DiskMessage};
use std::sync::Arc;
use tokio::fs::{File,OpenOptions};
use crate::io::{expand,write_chunk};

const ERROR:bool = true;
const DEBUG:bool = false;

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
            debug_error("failed-open_file-disk.rs",ERROR);
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
                debug_error("failed-receive_message-disk.rs",ERROR);
                break;
            }
        }

        match message{
            DiskMessage::Expand(value)=>{
                handle_expand(&mut config,value,&mut file).await;
            },
            DiskMessage::Add(value)=>{
                handle_add(value,&mut file).await;
            }
        }

    }

}

async fn handle_add(message:(u64,Vec<u8>,Arc<Mutex<Signal>>,Arc<Notify>),file:&mut File){
    debug_message("disk add message received",DEBUG);
    match write_chunk(file, message.0, message.1).await{
        Ok(_)=>{
            Signal::ok(message.2).await;
            debug_message("added to disk",DEBUG);
        },
        Err(_)=>{
            debug_message("failed add to disk",DEBUG);
            debug_error("failed-handle_add-disk.rs",ERROR);
        }
    }
    message.3.notify_waiters();
    debug_message("handle_add notified",DEBUG);
}

async fn handle_expand(config:&mut DiskConfig,message:(Arc<Mutex<Signal>>,Arc<Notify>),file:&mut File){
    match expand(file,&config.frame_size).await{
        Ok(_)=>{
            Signal::ok(message.0).await;
        },
        Err(_)=>{
            debug_error("failed-handle_expand-disk.rs",ERROR);
        }
    }
    message.1.notify_waiters();
}