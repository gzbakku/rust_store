use std::sync::Arc;
use crate::workers::Signal;
// use crate::workers::{debug_error,debug_message};
// use crate::workers::Debugger;
use crate::config::{MapConfig,MapMessage,DiskMessage,MapAddMessage,DiskAddMessage};
use tokio::sync::{Notify};
use gzb_binary_69::parser::writer::init as BinaryWriter;
// use std::time::Instant;

// const DEBUG:bool = false;
// const ERROR:bool = true;

pub async fn init(config:MapConfig){

    let mut config = config;

    loop{

        // let hold_time = Instant::now();

        let message:MapMessage;
        match config.receiver.recv_async().await{
            Ok(m)=>{
                message = m;
            },
            Err(_)=>{
                // debug_error("failed-receive_message-map.rs",ERROR);
                break;
            }
        }

        // let hold_after = Instant::now();

        match message{
            MapMessage::Add(value)=>{
                handle_add(&mut config,value).await;
            },
            MapMessage::Print=>{
                gzb_binary_69::workers::print_last_100_pointers(&mut config.reader);
            }
        }

        // println!("message in : {:?}",hold_after.elapsed());

        // println!("mapped in : {:?} {:?}",hold_time.elapsed(),hold_after.elapsed());

    }

}

async fn handle_add(config:&mut MapConfig,message:MapAddMessage){

    // let hold_time = Instant::now();

    // debug_message("map add message received",DEBUG);

    //Debuggerupdate(&message.debugger, "map add message received").await;

    //find the biggest value
    let biggest_value:u64;
    if config.items.len() == 0{
        biggest_value = 1;
    } else{
        biggest_value = config.items[config.items.len()-1]+1;
    }

    // println!("key in : {:?}",hold_time.elapsed());

    config.items.push(biggest_value);

    let notify = message.notify.clone();
    let value_len = message.value.len();
    let build_message_body:Vec<u8>;
    // let hold_time = Instant::now();
    match BinaryWriter(biggest_value.to_be_bytes().to_vec(),message.value){
        Ok(v)=>{
            build_message_body = v;
        },
        Err(_)=>{
            message.notify.notify_one();
            //Debuggererror(&message.debugger, "failed-parse-message-map.rs").await;
            // debug_error("failed-parse-message-map.rs",ERROR);
            return;
        }
    }
    // println!("parsed in : {:?}",hold_time.elapsed());

    // let mut index:u32 = 0;

    loop{

        // if index == 10000{
        //     // debug_message("overflow",DEBUG);
        //     notify.notify_one();
        //     //Debuggererror(&message.debugger, "overflow-send-message-map.rs").await;
        //     // debug_error("overflow-send-message-map.rs",ERROR);
        //     break;
        // }

        match config.reader.fill(biggest_value.to_be_bytes().to_vec(),value_len){
            Ok(write)=>{
                // println!("filled in : {:?}",hold_time.elapsed());
                // debug_message("added to map",DEBUG);
                match config.disk_sender.send_async(DiskMessage::Add(
                    /*
                        (
                            write.start as u64,
                            build_message_body,
                            message.signal,
                            message.notify
                        )
                    */
                    DiskAddMessage{
                        // debugger:message.debugger.clone(),
                        start_at:write.start as u64,
                        value:build_message_body,
                        signal:message.signal,
                        notify:message.notify
                    }
                )).await{
                    Ok(_)=>{
                        //Debuggerupdate(&message.debugger, "disk add message sent").await;
                        // debug_message("disk add message sent",DEBUG);
                    },
                    Err(_)=>{
                        //Debuggererror(&message.debugger, "failed-send-message-map.rs").await;
                        // debug_error("failed-send-message-map.rs",ERROR);
                        notify.notify_one();
                    }
                }
                // println!("sent in : {:?}",hold_time.elapsed());
                break;
            },
            Err(e)=>{
                if e == "full"{
                    //expand map
                    match config.reader.expand(config.frame_size.clone() as usize){
                        Ok(_)=>{},
                        Err(_)=>{
                            //Debuggererror(&message.debugger, "failed-expand-reader-map.rs").await;
                            // debug_error("failed-expand-reader-map.rs",ERROR);
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
                            //Debuggererror(&message.debugger, "failed-send_expand_message-map.rs").await;
                            // debug_error("failed-send_expand_message-map.rs",ERROR);
                            notify.notify_one();
                            break;
                        }
                    }
                    sleeper.notified().await;
                    if !Signal::check(signal).await{
                        //Debuggererror(&message.debugger, "failed-expand-disk-map.rs").await;
                        // debug_error("failed-expand-disk-map.rs",ERROR);
                        notify.notify_one();
                        break;
                    }
                } else {
                    //Debuggererror(&message.debugger, "failed-reader-fill-map.rs").await;
                    // debug_error("failed-reader-fill-map.rs",ERROR);
                    notify.notify_one();
                    break;
                }
            }//error
        }//match fill

        // index += 1;

    }//loop reader fill and expand

    // println!("added in : {:?}",hold_time.elapsed());

}

