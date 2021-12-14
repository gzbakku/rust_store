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
use crate::workers::{Signal,SignalData};
use tokio::sync::{Notify};
use std::sync::Arc;
use crate::config::{MapConfig,MapMessage,DiskConfig,MapAddMessage,MapGetMessage,MapRemoveMessage,MapResetMessage};

// use tokio::runtime::Builder as TokioRuntimeBuilder;

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
        let waker = Arc::new(Notify::new());
        let sleeper = waker.clone();

        match self.sender.send_async(
            MapMessage::Add(MapAddMessage{
                value:value,
                signal:signal.clone(),
                notify:waker
            }) 
        ).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err(());
            }
        }

        sleeper.notified().await;

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
    pub async fn get(&mut self)->Result<(Vec<u8>,u64),&'static str>{

        let signal = SignalData::new();
        let waker = Arc::new(Notify::new());
        let sleeper = waker.clone();

        match self.sender.send_async(
            MapMessage::Get(MapGetMessage{
                signal:signal.clone(),
                notify:waker
            }) 
        ).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err("failed-send-map-message");
            }
        }

        sleeper.notified().await;

        let hold = signal.lock().await;
        if !hold.result{
            return Err("failed-find_in-map");
        }
        return Ok((hold.data.clone(),hold.index.clone()));

    }
    pub async fn remove(&mut self,index:u64)->Result<(),&'static str>{

        let signal = Signal::new();
        let waker = Arc::new(Notify::new());
        let sleeper = waker.clone();

        match self.sender.send_async(
            MapMessage::Remove(MapRemoveMessage{
                index:index,
                signal:signal.clone(),
                notify:waker
            }) 
        ).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err("failed-send-map-message");
            }
        }

        sleeper.notified().await;

        if Signal::check(signal).await{
            return Ok(());
        } else {
            return Err("failed-listen-signal");
        }

    }
    pub async fn reset(&mut self,index:u64)->Result<(),&'static str>{

        let signal = Signal::new();
        let waker = Arc::new(Notify::new());
        let sleeper = waker.clone();

        match self.sender.send_async(
            MapMessage::Reset(MapResetMessage{
                index:index,
                signal:signal.clone(),
                notify:waker
            }) 
        ).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err("failed-send-map-reset-message");
            }
        }

        sleeper.notified().await;

        if !Signal::check(signal).await{
            return Err("failed-find_in-map");
        }
        return Ok(());

    }
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