
use crate::workers::Signal;
use tokio::sync::{Notify,Mutex};
use crate::config::{DiskConfig,DiskMessage};
use std::sync::Arc;
use tokio::fs::{File,OpenOptions};
use crate::io::{expand};
// use gzb_binary_69::Write;

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
                break;
            }
        }

        match message{
            DiskMessage::Expand(value)=>{
                handle_expand(&mut config,value,&mut file).await;
            },
            DiskMessage::Add(value)=>{
                handle_add(&mut config,value,&mut file).await;
            }
        }

    }

}

async fn handle_add(config:&mut DiskConfig,value:(Vec<u8>,Arc<Mutex<Signal>>,Arc<Notify>),file:&mut File){



}

async fn handle_expand(config:&mut DiskConfig,value:(Arc<Mutex<Signal>>,Arc<Notify>),file:&mut File){
    match expand(file,&config.frame_size).await{
        Ok(_)=>{
            Signal::ok(value.0).await;
        },
        Err(_)=>{}
    }
    value.1.notify_waiters();
}