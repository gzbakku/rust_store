use std::sync::Arc;
use crate::workers::Signal;
// use crate::workers::{debug_error,debug_message};
// use crate::workers::Debugger;
use crate::config::{
    MapConfig,MapMessage,DiskMessage,MapAddMessage,
    DiskAddMessage,MapGetMessage,DiskGetMessage,
    MapRemoveMessage,DiskRemoveMessage,MapResetMessage
};
use tokio::sync::{Notify};
use gzb_binary_69::parser::writer::init as BinaryWriter;
// use std::time::Instant;

// const DEBUG:bool = false;
// const ERROR:bool = true;

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
            },
            MapMessage::AddUn(value)=>{
                handle_add_unchecked(&mut config,value).await;
            },
            MapMessage::Get(value)=>{
                handle_get(&mut config,value).await;
            }
            MapMessage::Remove(value)=>{
                handle_remove(&mut config,value).await;
            },
            MapMessage::Reset(value)=>{
                handle_reset(&mut config,value).await;
            }
        }

    }

}

async fn handle_add(config:&mut MapConfig,message:MapAddMessage){

    //find the biggest value
    let biggest_value:u64;
    if config.items.len() == 0{
        biggest_value = 1;
    } else{
        biggest_value = config.items[config.items.len()-1]+1;
    }

    config.items.push(biggest_value);

    let notify = message.notify.clone();
    let value_len = message.value.len();
    let build_message_body:Vec<u8>;
    match BinaryWriter(biggest_value.to_be_bytes().to_vec(),message.value){
        Ok(v)=>{
            build_message_body = v;
        },
        Err(_)=>{
            message.notify.notify_one();
            return;
        }
    }

    loop{

        match config.reader.fill(biggest_value.to_be_bytes().to_vec(),value_len){
            Ok(write)=>{
                match config.disk_sender.send_async(DiskMessage::Add(
                    DiskAddMessage{
                        // debugger:message.debugger.clone(),
                        start_at:write.start as u64,
                        value:build_message_body,
                        signal:message.signal,
                        notify:message.notify
                    }
                )).await{
                    Ok(_)=>{},
                    Err(_)=>{
                        notify.notify_one();
                    }
                }
                break;
            },
            Err(e)=>{
                if e == "full"{
                    //expand map
                    match config.reader.expand(config.frame_size.clone() as usize){
                        Ok(_)=>{},
                        Err(_)=>{
                            notify.notify_one();
                            break;
                        }
                    }
                    //expand disk
                    let signal = Signal::new();
                    let waker = Arc::new(Notify::new());
                    let sleeper = waker.clone();
                    match config.disk_sender.send_async(DiskMessage::Expand((signal.clone(),waker))).await{
                        Ok(_)=>{},
                        Err(_)=>{
                            notify.notify_one();
                            break;
                        }
                    }
                    sleeper.notified().await;
                    if !Signal::check(signal).await{
                        notify.notify_one();
                        break;
                    }
                } else {
                    notify.notify_one();
                    break;
                }
            }//error
        }//match fill

    }//loop reader fill and expand

    // println!("added in : {:?}",hold_time.elapsed());

}

async fn handle_add_unchecked(config:&mut MapConfig,message:Vec<u8>){

    //find the biggest value
    let biggest_value:u64;
    if config.items.len() == 0{
        biggest_value = 1;
    } else{
        biggest_value = config.items[config.items.len()-1]+1;
    }

    config.items.push(biggest_value);

    let value_len = message.len();
    let build_message_body:Vec<u8>;
    match BinaryWriter(biggest_value.to_be_bytes().to_vec(),message){
        Ok(v)=>{
            build_message_body = v;
        },
        Err(_)=>{
            return;
        }
    }

    loop{

        match config.reader.fill(biggest_value.to_be_bytes().to_vec(),value_len){
            Ok(write)=>{
                match config.disk_sender.send_async(DiskMessage::AddUn((write.start as u64,build_message_body))).await{
                    Ok(_)=>{},
                    Err(_)=>{}
                }
                break;
            },
            Err(e)=>{
                if e == "full"{
                    //expand map
                    match config.reader.expand(config.frame_size.clone() as usize){
                        Ok(_)=>{},
                        Err(_)=>{
                            break;
                        }
                    }
                    //expand disk
                    let signal = Signal::new();
                    let waker = Arc::new(Notify::new());
                    let sleeper = waker.clone();
                    match config.disk_sender.send_async(DiskMessage::Expand((signal.clone(),waker))).await{
                        Ok(_)=>{},
                        Err(_)=>{
                            break;
                        }
                    }
                    sleeper.notified().await;
                    if !Signal::check(signal).await{
                        break;
                    }
                } else {
                    break;
                }
            }//error
        }//match fill

    }//loop reader fill and expand

}

async fn handle_remove(config:&mut MapConfig,message:MapRemoveMessage){

    let boundry:(usize,usize);
    let key = message.index.to_be_bytes().to_vec();
    match config.reader.pointers.get(&key){
        Some(pointer)=>{
            boundry = pointer.0;
        },
        None=>{
            message.notify.notify_one();
            return;
        }
    }

    match config.reader.clear(&key){
        Ok(_)=>{},
        Err(_)=>{
            message.notify.notify_one();
            return;
        }
    }

    let notify = message.notify.clone();
    match config.disk_sender.send_async(DiskMessage::Remove(
        DiskRemoveMessage{
            boundry:boundry,
            signal:message.signal,
            notify:message.notify
        }
    )).await{
        Ok(_)=>{},
        Err(_)=>{
            notify.notify_one();
        }
    }

}

async fn handle_get(config:&mut MapConfig,message:MapGetMessage){

    if config.items.len() == 0{
        message.notify.notify_one();
        return;
    }

    let index:u64 = config.items.remove(0);
    let notify = message.notify.clone();
    config.items_in_processing.push(index);

    let value_boundry:(usize,usize);
    match config.reader.pointers.get(
        &index.to_be_bytes().to_vec()
    ){
        Some(pointer)=>{
            value_boundry = pointer.1;
        },
        None=>{
            notify.notify_one();
            return;
        }
    }

    match config.disk_sender.send_async(DiskMessage::Get(
        DiskGetMessage{
            index:index,
            value_boundry:value_boundry,
            signal:message.signal,
            notify:message.notify
        }
    )).await{
        Ok(_)=>{},
        Err(_)=>{
            notify.notify_one();
        }
    }

}

async fn handle_reset(config:&mut MapConfig,message:MapResetMessage){

    if config.items_in_processing.len() == 0{
        message.notify.notify_one();
        return;
    }

    let index:usize;
    match config.items_in_processing.iter().position(|&x| x == message.index){
        Some(i)=>{
            index = i;
        },
        None=>{
            message.notify.notify_one();
        return;
        }
    }

    let hold = config.items_in_processing.remove(index);
    config.items.push(hold);
    config.items.sort();
    Signal::ok(message.signal).await;
    message.notify.notify_one();
    return;

}

