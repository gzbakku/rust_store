use std::sync::Arc;
use crate::workers::{Signal,debug_error,debug_message};
use crate::config::{MapConfig,MapMessage,DiskMessage};
use tokio::sync::{Notify,Mutex};
use gzb_binary_69::parser::writer::init as BinaryWriter;

const DEBUG:bool = false;
const ERROR:bool = true;

pub async fn init(config:MapConfig){

    let mut config = config;

    loop{

        let message:MapMessage;
        match config.receiver.recv_async().await{
            Ok(m)=>{
                message = m;
            },
            Err(_)=>{
                debug_error("failed-receive_message-map.rs",ERROR);
                break;
            }
        }

        match message{
            MapMessage::Add(value)=>{
                handle_add(&mut config,value).await;
            },
            MapMessage::Print=>{
                gzb_binary_69::workers::print_last_100_pointers(&mut config.reader);
            }
        }

    }

}

async fn handle_add(config:&mut MapConfig,message:(Vec<u8>,Arc<Mutex<Signal>>,Arc<Notify>)){

    debug_message("map add message received",DEBUG);

    //find the biggest value
    let biggest_value:u64;
    if config.items.len() == 0{
        biggest_value = 1;
    } else{
        biggest_value = config.items[config.items.len()-1]+1;
    }

    config.items.push(biggest_value);

    let notify_hold = message.2.clone();
    let value_len = message.0.len();
    let build_message_body:Vec<u8>;
    match BinaryWriter(biggest_value.to_be_bytes().to_vec(),message.0){
        Ok(v)=>{
            build_message_body = v;
        },
        Err(_)=>{
            message.2.notify_waiters();
            debug_error("failed-parse-message-map.rs",ERROR);
            return;
        }
    }

    let mut index = 0;

    loop{

        let notify = notify_hold.clone();

        if index == 10000{
            // debug_message("overflow",DEBUG);
            notify.notify_waiters();
            debug_error("overflow-send-message-map.rs",ERROR);
            break;
        }

        match config.reader.fill(biggest_value.to_be_bytes().to_vec(),value_len){
            Ok(write)=>{
                // debug_message("added to map",DEBUG);
                match config.disk_sender.send_async(DiskMessage::Add((
                    write.start as u64,
                    build_message_body,
                    message.1,
                    message.2
                ))).await{
                    Ok(_)=>{
                        debug_message("disk add message sent",DEBUG);
                    },
                    Err(_)=>{
                        debug_error("failed-send-message-map.rs",ERROR);
                        notify.notify_waiters();
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
                            debug_error("failed-expand-reader-map.rs",ERROR);
                            notify.notify_waiters();
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
                            debug_error("failed-send_expand_message-map.rs",ERROR);
                            notify.notify_waiters();
                            break;
                        }
                    }
                    sleeper.notified().await;
                    if !Signal::check(signal).await{
                        debug_error("failed-expand-disk-map.rs",ERROR);
                        notify.notify_waiters();
                        break;
                    }
                } else {
                    debug_error("failed-reader-fill-map.rs",ERROR);
                    notify.notify_waiters();
                    break;
                }
            }//error
        }//match fill

        index += 1;

    }//loop reader fill and expand

}

