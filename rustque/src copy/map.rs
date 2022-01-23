// use std::sync::Arc;
use crate::workers::Signal;
// use crate::workers::{debug_error,debug_message};
// use crate::workers::Debugger;
use crate::config::{
    MapConfig,MapMessage,
    MapRemoveMessage,MapAddMessage,MapGetMessage,
    DiskConfig,DiskMessage,DiskAddMessage,DiskGetMessage,DiskRemoveMessage
};
// use tokio::sync::{Notify};
use gzb_binary_69::parser::writer::init as BinaryWriter;
use tokio::spawn as TokioSpawn;
use flume::unbounded;
use flume::Sender as FlumeSender;
// use crate::workers::{debug_error};
// use std::time::Instant;

// const DEBUG:bool = false;
// const ERROR:bool = true;

pub async fn init(config:MapConfig){

    let (mut disk_sender,disk_receiver) = unbounded();
    let disk_config = DiskConfig::new(
        config.file_path.clone(),
        config.min_que_size.clone(),
        disk_receiver
    );
    for _ in 0..config.num_of_writers{
        let hold_disk_config = disk_config.clone();
        TokioSpawn(async move {
            crate::disk::init(hold_disk_config).await;
        });
    }

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
                handle_add(&mut config,&mut disk_sender,value).await;
            },
            MapMessage::Get(value)=>{
                handle_get(&mut config,&mut disk_sender,value).await;
            },
            MapMessage::Remove(value)=>{
                handle_remove(&mut config,&mut disk_sender,value).await;
            }
        }

    }

}

async fn handle_add(
    config:&mut MapConfig,
    disk_sender:&mut FlumeSender<DiskMessage>,
    message:MapAddMessage
){

    let value_len:usize = message.value.len();
    let build_message_body:Vec<u8>;
    match BinaryWriter(
        message.index.to_be_bytes().to_vec(),
        message.value
    ){
        Ok(v)=>{
            build_message_body = v;
        },
        Err(_)=>{
            // println!("!!!");
            Signal::error(message.signal).await;
            return;
        }
    }

    loop{

        match config.reader.fill(message.index.to_be_bytes().to_vec(),value_len.clone()){
            Ok(write)=>{
                match disk_sender.send_async(DiskMessage::Add(
                    DiskAddMessage{
                        // debugger:message.debugger.clone(),
                        start_at:write.start as u64,
                        value:build_message_body,
                        signal:message.signal.clone()
                    }
                )).await{
                    Ok(_)=>{
                        // debug_message("filled_passed-add-map",DEBUG);
                    },
                    Err(_)=>{
                        // debug_error("failed-send_disk_message-add-map",ERROR);
                        Signal::error(message.signal).await;
                    }
                }
                break;
            },
            Err(e)=>{
                if e == "full"{
                    //expand map
                    match config.reader.expand(config.expand_size.clone() as usize){
                        Ok(_)=>{},
                        Err(_)=>{
                            // debug_error("failed-reader_expand-add-map",ERROR);
                            Signal::error(message.signal).await;
                            break;
                        }
                    }
                    //expand disk
                    let (signal,sleeper) = Signal::new();
                    match disk_sender.send_async(DiskMessage::Expand(signal.clone())).await{
                        Ok(_)=>{},
                        Err(_)=>{
                            // debug_error("failed-send_expand_message-add-map",ERROR);
                            Signal::error(message.signal).await;
                            break;
                        }
                    }
                    sleeper.notified().await;
                    if !Signal::check(&signal).await{
                        // debug_error("failed-expand_disk-add-map",ERROR);
                        Signal::error(message.signal).await;
                        break;
                    }
                } else {
                    // debug_error("failed-reader_fill-add-map",ERROR);
                    Signal::error(message.signal).await;
                    break;
                }
            }//error
        }//match fill

    }//loop reader fill and expand

    // println!("added in : {:?}",hold_time.elapsed());

}

async fn handle_remove(
    config:&mut MapConfig,
    disk_sender:&mut FlumeSender<DiskMessage>,
    message:MapRemoveMessage
){

    let boundry:(usize,usize);
    let key = message.pointer.item_index.to_be_bytes().to_vec();
    match config.reader.pointers.get(&key){
        Some(pointer)=>{
            boundry = pointer.0;
        },
        None=>{
            Signal::error(message.signal).await;
            return;
        }
    }

    match config.reader.clear(&key){
        Ok(_)=>{},
        Err(_)=>{
            Signal::error(message.signal).await;
            return;
        }
    }

    match disk_sender.send_async(DiskMessage::Remove(
        DiskRemoveMessage{
            boundry:boundry,
            signal:message.signal.clone()
        }
    )).await{
        Ok(_)=>{},
        Err(_)=>{
            Signal::error(message.signal).await;
        }
    }

}

async fn handle_get(
    config:&mut MapConfig,
    disk_sender:&mut FlumeSender<DiskMessage>,
    message:MapGetMessage
){

    let value_boundry:(usize,usize);
    match config.reader.pointers.get(
        &message.index.to_be_bytes().to_vec()
    ){
        Some(pointer)=>{
            value_boundry = pointer.1;
        },
        None=>{
            Signal::error(message.signal).await;
            return;
        }
    }

    match disk_sender.send_async(DiskMessage::Get(
        DiskGetMessage{
            item_index:message.index,
            map_index:config.map_index.clone(),
            value_boundry:value_boundry,
            signal:message.signal.clone()
        }
    )).await{
        Ok(_)=>{},
        Err(_)=>{
            Signal::error(message.signal).await;
            return;
        }
    }

}
