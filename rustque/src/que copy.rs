use crate::io;
use crate::Config;
use crate::map::init as MapSpawnInit;
use crate::disk::init as DiskSpawnInit;
use std::fs::Metadata;
use gzb_binary_69::{Reader};
use tokio::fs::File;
use tokio::spawn as TokioSpawn;
use flume::Sender as FlumeSender;
use crate::workers::{u64_from_bytes};
use crate::workers::Signal;
use tokio::sync::Notify;
use std::sync::Arc;
use crate::config::{MapConfig,MapMessage,DiskConfig,MapAddMessage};

#[derive(Debug,Clone)]
pub struct Que{
    sender:FlumeSender<MapMessage>
}

const MB1:u64 = 10_000;

impl Que{
    pub async fn new(c:Config)->Result<Que,&'static str>{

        //ensure file 
        let metadata:Metadata;
        let mut file:File;
        match io::init_map(c.path.clone(),c.frame).await{
            Ok(v)=>{
                file = v.0;
                metadata = v.1;
            },
            Err(_)=>{
                //debug_error("failed-init-map-que.rs",ERROR);
                return Err("failed-init-map");
            }
        }

        let reader:Reader;
        let que_items:Vec<u64>;
        match build_map(metadata,&mut file).await{
            Ok(r)=>{
                reader = r.0;
                que_items = r.1;
            },
            Err(_)=>{
                //debug_error("failed-build-map-que.rs",ERROR);
                return Err("failed-build-map");
            }
        }

        //disk config
        let (disk_config,disk_sender) = DiskConfig::new(c.path,c.frame.clone());
        for _ in 0..c.disk_writers{
            let hold_config = disk_config.clone();
            TokioSpawn(async move {
                DiskSpawnInit(hold_config).await
            });
        }

        //build mpsc channel for que->map
        let (map_config,map_sender) = MapConfig::new(reader,disk_sender,que_items,c.frame.clone());
        TokioSpawn(async move {
            MapSpawnInit(map_config).await
        });

        //build que
        return Ok(Que{
            sender:map_sender
        });

    }
    pub async fn add(&mut self,value:Vec<u8>)->Result<(),()>{

        let signal = Signal::new();
        // let debugger = Debugger::new();
        let waker = Arc::new(Notify::new());
        let sleeper = waker.clone();

        //debug_message("\nadding",DEBUG);

        //Debugger::update(&debugger, "adding").await;

        match self.sender.send_async(
            MapMessage::Add(MapAddMessage{
                // debugger:debugger.clone(),
                value:value,
                signal:signal.clone(),
                notify:waker
            }) /*(value,signal.clone(),waker))*/
        ).await{
            Ok(_)=>{
                //Debugger::update(&debugger, "map add message sent").await;
                //debug_message("map add message sent",DEBUG);
            },
            Err(_)=>{
                //debug_error("failed-send_add_message-que.rs",ERROR);
                return Err(());
            }
        }

        // sleeper.notified().await;

        //debug_message("listening for notification",DEBUG);
        //Debugger::update(&debugger, "listening for notification").await;

        sleeper.notified().await;

        //debug_message("noti received",DEBUG);
        //Debugger::update(&debugger, "noti received").await;

        if Signal::check(signal).await{
            return Ok(());
        } else {
            return Err(());
        }

    }
    pub async fn add_unchecked(&mut self,value:Vec<u8>)->Result<(),()>{
        match self.sender.send_async(MapMessage::AddUn(value)).await{
            Ok(_)=>{
                return Ok(());
            },
            Err(_)=>{
                return Err(());
            }
        }
    }//add unchecked
}

async fn build_map(metadata:Metadata,file:&mut File)->Result<(Reader,Vec<u64>),&'static str>{

    //meta info
    let mut len:u64 = metadata.len();//bytes
    let chunk_size:u64; //bytes
    if len > MB1{
        chunk_size = MB1;
    } else {
        chunk_size = len;
    }

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
                reader.map(&mut read_buffer);
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