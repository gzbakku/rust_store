use crate::workers::{Signal};
// use crate::workers::{debug_error,debug_message};
// use crate::workers::Debugger;
// use tokio::sync::{Notify,Mutex};
use crate::config::{LocatorMessage};
use crate::config::{MapMessage,MapGetMessage,MapAddMessage,MapRemoveMessage};
// use std::sync::Arc;
use std::collections::HashMap;
use flume::Receiver;
use flume::Sender as FlumeSender;
// use std::time::Instant;

// const ERROR:bool = true;
// const DEBUG:bool = false;

pub async fn init(
    map_senders:&mut HashMap<u8,FlumeSender<MapMessage>>,
    items:&mut Vec<u64>,
    locator:&mut HashMap<u64,u8>,
    receiver:Receiver<LocatorMessage>,
    total_maps:u8
){

    // let mut receiver = receiver;
    let mut biggest:u64 = 1;
    let mut last_map_added = 1;
    if items.len() > 0{
        biggest = items[items.len()-1].clone();
    }

    loop{

        match receiver.recv_async().await{
            Ok(m)=>{
                match m{
                    LocatorMessage::Next(message)=>{
                        if items.len() ==0{
                            Signal::error(message.signal).await;
                        } else {
                            let item_index = items.remove(0);
                            match locator.remove(&item_index){
                                Some(map_index)=>{
                                    match map_senders.get_mut(&map_index){
                                        Some(sender)=>{
                                            match sender.send_async(MapMessage::Get(MapGetMessage{
                                                index:item_index,
                                                signal:message.signal.clone()
                                            })).await{
                                                Ok(_)=>{},
                                                Err(_)=>{
                                                    Signal::error(message.signal).await;
                                                }
                                            }
                                        },
                                        None=>{
                                            Signal::error(message.signal).await;
                                        }
                                    }
                                },
                                None=>{
                                    Signal::error(message.signal).await;
                                }
                            }
                        }
                    },
                    LocatorMessage::Add(message)=>{
                        //find next biggest index
                        if items.len() == 0{biggest = 1;} else {biggest += 1;}
                        if last_map_added == total_maps{last_map_added = 1;} else {last_map_added += 1;}
                        locator.insert(biggest.clone(),last_map_added.clone());
                        items.push(biggest);
                        match map_senders.get_mut(&last_map_added){
                            Some(sender)=>{
                                match sender.send_async(MapMessage::Add(MapAddMessage{
                                    index:biggest,
                                    value:message.value,
                                    signal:message.signal.clone()
                                })).await{
                                    Ok(_)=>{
                                        // debug_message("sent_message-add-locator", DEBUG);
                                    },
                                    Err(_)=>{
                                        // debug_error("failed-send_map_message-add-locator", ERROR);
                                        Signal::error(message.signal).await;
                                    }
                                }
                            },
                            None=>{
                                // debug_error("failed-get_mut_locator-add-locator", ERROR);
                                Signal::error(message.signal).await;
                            }
                        }
                    },
                    LocatorMessage::Remove(message)=>{
                        match map_senders.get_mut(&message.pointer.map_index){
                            Some(sender)=>{
                                match sender.send_async(MapMessage::Remove(MapRemoveMessage{
                                    pointer:message.pointer,
                                    signal:message.signal.clone()
                                })).await{
                                    Ok(_)=>{},
                                    Err(_)=>{
                                        Signal::error(message.signal).await;
                                    }
                                }
                            },
                            None=>{
                                Signal::error(message.signal).await;
                            }
                        }
                    }
                    LocatorMessage::Reset(message)=>{
                        items.push(message.pointer.item_index.clone());
                        locator.insert(
                            message.pointer.item_index.clone(),
                            message.pointer.map_index.clone()
                        );
                        Signal::ok(message.signal).await;
                    }
                }
            },
            Err(_e)=>{
                println!("!!! failed-locator-receive : {:?}",_e);
                // debug_error("failed-receive_message-disk.rs",ERROR);
                break;
            }
        }

    }

    // panic!("locator crashed");

}