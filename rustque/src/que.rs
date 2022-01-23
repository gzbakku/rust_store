use crate::io;
use crate::Config;
// use crate::map::init as MapSpawnInit;
// use crate::disk::init as DiskSpawnInit;
use std::fs::Metadata;
use gzb_binary_69::{Reader};
use tokio::fs::File;
use tokio::spawn as TokioSpawn;
use flume::Sender as FlumeSender;
// use flume::Receiver as FlumeReceiver;
use crate::workers::{u64_from_bytes};
use crate::workers::{Signal,Pointer};
// use tokio::sync::{Notify,Mutex};
use std::sync::Arc;
use crate::config::{
    MapConfig,MapMessage,
    // MapAddMessage,MapGetMessage,MapRemoveMessage,
    LocatorMessage,LocatorNext,LocatorAdd,LocatorRemove,LocatorReset
    // DiskConfig,
};
use flume::unbounded;
use std::collections::HashMap;

use futures::future::join_all;
use crate::response::QueResponse;

// use tokio::runtime::Builder as TokioRuntimeBuilder;

#[derive(Debug,Clone)]
pub struct Que{
    locator_sender:Arc<FlumeSender<LocatorMessage>>
}

const MB_1:u64 = 1_000_000;

impl Que{
    pub async fn new(c:Config)->Result<Que,&'static str>{

        // println!("que config : {:?}",c);

        let (locator_sender,locator_receiver) = unbounded();
        let mut locator:HashMap<u64,u8> = HashMap::new();
        let mut all_items = Vec::new();

        let mut map_index:u8 = 0;
        let mut collect_map_initiaters = Vec::new();
        for path in c.files.iter(){
            map_index += 1;
            collect_map_initiaters.push(init_map(
                map_index,
                path.clone(),
                locator_sender.clone(),
                c.num_of_writers.clone(),
                c.min_que_size.clone(),
                c.expand_size.clone()
            ));
        }

        let mut map_senders = HashMap::new();
        for result in join_all(collect_map_initiaters).await{
            match result{
                Ok((index,mut items,map_sender))=>{
                    for item in items.iter(){
                        locator.insert(item.clone(),index);
                    }
                    all_items.append(&mut items);
                    map_senders.insert(index,map_sender);
                },
                Err(_e)=>{
                    return Err(_e);
                }
            }
        }

        all_items.sort();
        // println!("{:?}",all_items.len());
        TokioSpawn(async move{
            crate::locator::init(
                &mut map_senders,
                &mut all_items,
                &mut locator,
                locator_receiver,
                map_index
            ).await;
        });

        return Ok(Que{
            locator_sender:Arc::new(locator_sender)
        });

    }
    pub async fn add(&mut self,value:Vec<u8>)->Result<QueResponse,()>{

        let (signal,sleeper) = Signal::new();
        match self.locator_sender.send_async(
            LocatorMessage::Add(LocatorAdd{
                value:value,
                signal:signal.clone(),
            })
        ).await{
            Ok(_)=>{
                return Ok(QueResponse::new(signal,sleeper));
            },
            Err(_)=>{
                return Err(());
            }
        }

    }
    pub async fn next(&mut self)->Result<QueResponse,()>{

        let (signal,sleeper) = Signal::new();
        match self.locator_sender.send_async(
            LocatorMessage::Next(LocatorNext{
                signal:signal.clone(),
            })
        ).await{
            Ok(_)=>{
                return Ok(QueResponse::new(signal,sleeper));
            },
            Err(_)=>{
                return Err(());
            }
        }

    }
    pub async fn remove(&mut self,pointer:Pointer)->Result<QueResponse,()>{

        let (signal,sleeper) = Signal::new();
        match self.locator_sender.send_async(
            LocatorMessage::Remove(LocatorRemove{
                pointer:pointer,
                signal:signal.clone(),
            })
        ).await{
            Ok(_)=>{
                return Ok(QueResponse::new(signal,sleeper));
            },
            Err(_)=>{
                return Err(());
            }
        }

    }
    pub async fn reset(&mut self,pointer:Pointer)->Result<QueResponse,()>{

        let (signal,sleeper) = Signal::new();
        match self.locator_sender.send_async(
            LocatorMessage::Reset(LocatorReset{
                pointer:pointer,
                signal:signal.clone(),
            })
        ).await{
            Ok(_)=>{
                return Ok(QueResponse::new(signal,sleeper));
            },
            Err(_)=>{
                return Err(());
            }
        }

    }
}

async fn init_map(
    index:u8,
    path:String,
    locator_sender:FlumeSender<LocatorMessage>,
    num_of_writers:u8,
    min_que_size:u64,
    expand_size:u64
)->Result<(u8,Vec<u64>,FlumeSender<MapMessage>),&'static str>{

    //ensure file 
    let metadata:Metadata;
    let mut file:File;
    match io::init_map(path.clone(),min_que_size).await{
        Ok(v)=>{
            file = v.0;
            metadata = v.1;
        },
        Err(_e)=>{
            //debug_error("failed-init-map-que.rs",ERROR);
            println!("!!! failed-init-map-que : {:?}",_e);
            return Err("failed-init-map");
        }
    }

    let reader:Reader;
    let items:Vec<u64>;
    match build_map(metadata,&mut file).await{
        Ok(r)=>{
            reader = r.0;
            items = r.1;
            // return Ok((index,r.0,r.1));
        },
        Err(_e)=>{
            //debug_error("failed-build-map-que.rs",ERROR);
            println!("!!! failed-build-map : {:?}",_e);
            return Err("failed-build-map");
        }
    }

    // println!("{:?}",reader.corrupt);

    let (build_map_config,map_sender) = MapConfig::new(
        index.clone(),
        path,
        reader,
        locator_sender,
        num_of_writers,
        min_que_size,
        expand_size
    );

    TokioSpawn(async move{
        crate::map::init(build_map_config).await;
    });

    // return Err("no_error");

    return Ok((index,items,map_sender));

}

async fn build_map(metadata:Metadata,file:&mut File)->Result<(Reader,Vec<u64>),&'static str>{

    // println!("building map");

    //meta info
    let mut len:u64 = metadata.len();//bytes
    // println!("file len : {:?}",len);
    let chunk_size:u64; //bytes

    if len > (MB_1 * 100){
        chunk_size = 100 * MB_1;//println!("100");
    } else if len > (MB_1 * 50){
        chunk_size = 50 * MB_1;//println!("50");
    } else if len > (MB_1 * 25){
        chunk_size = 25 * MB_1;//println!("25");
    } else {
        chunk_size = len;//println!("0");
    }

    // println!("chunk_size : {:?}",chunk_size);

    //make map
    let mut reader:Reader = Reader::with_capacity(1000000, 1000000);
    

    //read chunks
    let mut index = 0;
    let mut read_buffer:Vec<u8> = Vec::with_capacity(chunk_size as usize);

    loop{
        if len == 0{
            break;
        }
        let start_at = index*chunk_size;
        let read_len:u64;
        if len < chunk_size{read_len = len;} else {read_len = chunk_size;}
        match io::read_chunk(file, &mut read_buffer, start_at, read_len).await{
            Ok(_)=>{
                // println!("read complete");
                reader.map(&mut read_buffer);
                // println!("map complete");
            },
            Err(_)=>{
                return Err("failed-read_chunk");
            }
        }
        if len > chunk_size{
            len -= chunk_size;
        } else {
            len = 0;
        }
        index += 1;
    }

    match reader.end(){
        Ok(_)=>{},
        Err(_)=>{}
    }

    let mut collect:Vec<u64> = Vec::with_capacity(reader.pointers.len());
    for key in reader.pointers.keys(){
        match u64_from_bytes(&key){
            Ok(num)=>{
                collect.push(num);
            },
            Err(_)=>{}
        }
    }

    collect.sort();
    return Ok((reader,collect));

}