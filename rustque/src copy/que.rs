use crate::io;
use crate::Config;
use crate::map::init as MapSpawnInit;
use crate::disk::init as DiskSpawnInit;
use std::fs::Metadata;
use gzb_binary_69::{Reader};
use tokio::fs::File;
use tokio::spawn as TokioSpawn;
use flume::Sender as FlumeSender;
use crate::workers::{u64_from_bytes,debug_error};
use crate::workers::Signal;
use tokio::sync::Notify;
use std::sync::Arc;
use crate::config::{MapConfig,MapMessage,DiskConfig};
use tokio::time::timeout;
use std::time::Duration;

const ERROR:bool = true;

#[derive(Debug,Clone)]
pub struct Que{
    sender:FlumeSender<MapMessage>
}

const MB1:u64 = 1000_1000;

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
                debug_error("failed-init-map-que.rs",ERROR);
                // println!("map init failed : {:?}",e);
                return Err("failed-init-map");
            }
        }

        let reader:Reader;
        let que_items:Vec<u64>;
        match build_map(metadata,&mut file).await{
            Ok(r)=>{
                // println!("{:?}",reader.map);
                reader = r.0;
                que_items = r.1;
            },
            Err(_)=>{
                debug_error("failed-build-map-que.rs",ERROR);
                return Err("failed-build-map");
            }
        }

        // println!("{:?}",reader.map);

        // gzb_binary_69::workers::print_pointers(&mut reader);

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

        match timeout(
            Duration::from_secs(1),
            self.sender.send_async(
                MapMessage::Add((value,signal.clone(),waker))
            )
        ).await{
            Ok(v)=>{
                match v{
                    Ok(_)=>{},
                    Err(_)=>{
                        debug_error("failed-send_add_message-que.rs",ERROR);
                        return Err(());
                    }
                }
            },
            Err(_)=>{
                debug_error("timeout-send_add_message-que.rs",ERROR);
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
    pub async fn print_map(&mut self){
        println!("where");
        match self.sender.send_async(MapMessage::Print).await{
            Ok(_)=>{
                println!("poker");
            },
            Err(_)=>{
                println!("joker");
                debug_error("failed-send_print_message-que.rs",ERROR);
            }
        }
    }
}

async fn build_map(metadata:Metadata,file:&mut File)->Result<(Reader,Vec<u64>),&'static str>{

    //meta info
    let mut len:u64 = metadata.len();//bytes
    let chunk_size:u64; //bytes
    if len > MB1*100{
        chunk_size = MB1*100;
    } else 
    if len > MB1*50{
        chunk_size = MB1*50;
    } else 
    if len > MB1*25{
        chunk_size = MB1*25;
    } else 
    if len > MB1*10{
        chunk_size = MB1*10;
    } else 
    if len > MB1*5{
        chunk_size = MB1*5;
    } else {
        chunk_size = len;
    }

    //make map
    let mut reader:Reader = Reader::new();

    //read chunks
    let mut index = 0;
    let mut read_buffer:Vec<u8> = Vec::with_capacity(chunk_size as usize);

    loop{
        if len == 0{break;}
        let start_at = index*chunk_size;
        let read_len:u64;
        if len > chunk_size{read_len = chunk_size;} else {read_len = len;}
        match io::read_chunk(file, &mut read_buffer, start_at, read_len).await{
            Ok(_)=>{
                // println!("\n{:?}\n",read_buffer);
                match reader.map(&mut read_buffer){
                    Ok(_)=>{},
                    Err(_)=>{}
                }
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

    let mut collect:Vec<u64> = Vec::new();
    for i in reader.map.iter(){
        match u64_from_bytes(&i.key.2){
            Ok(num)=>{
                let mut collect_index = 0;
                let mut smaller_found = false;
                for i in collect.iter(){
                    if &num < i{smaller_found = true;break;}
                    collect_index += 1;
                }
                if smaller_found{collect.insert(collect_index,num);} else {collect.push(num);}
            },
            Err(_)=>{}
        }
    }

    return Ok((reader,collect));

}