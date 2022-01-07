use gzb_binary_69::Reader;
// use flume::unbounded as FlumeUnBounded;
use flume::bounded as FlumeBounded;
use flume::Receiver as FlumeReveiver;
use flume::Sender as FlumeSender;
use crate::workers::{Signal,Pointer};
// use crate::workers::Debugger;
use std::sync::Arc;
use tokio::sync::{Mutex};

//-------------------------------
//primary config
//-------------------------------

#[derive(Clone,Debug)]
pub struct Config{
    pub files:Vec<String>,
    pub min_que_size:u64,
    pub expand_size:u64,
    pub num_of_writers:u8
}

impl Config{
    pub fn new(files:Vec<String>,min_que_size:u64,expand_size:u64,num_of_writers:u8)->Config{
        Config{
            files:files,
            min_que_size:min_que_size,
            expand_size:expand_size,
            num_of_writers:num_of_writers
        }
    }
}

//-------------------------------
//messages
//-------------------------------

//-------------------------------
//dis message
//-------------------------------

pub struct DiskGetMessage{
    pub item_index:u64,
    pub map_index:u8,
    pub value_boundry:(usize,usize),//(start,end)
    pub signal:Arc<Mutex<Signal>>
}

pub struct DiskAddMessage{
    // pub debugger:Arc<Mutex<Debugger>>,
    pub start_at:u64,
    pub value:Vec<u8>,
    pub signal:Arc<Mutex<Signal>>
}

pub struct DiskRemoveMessage{
    pub boundry:(usize,usize),//(start,end)
    pub signal:Arc<Mutex<Signal>>
}

pub enum DiskMessage{
    Expand(Arc<Mutex<Signal>>),
    Add(DiskAddMessage),
    Get(DiskGetMessage),
    Remove(DiskRemoveMessage)
}

#[derive(Clone)]
pub struct DiskConfig{
    pub receiver:FlumeReveiver<DiskMessage>,
    pub path:String,
    pub frame_size:u64
}

impl DiskConfig{
    pub fn new(p:String,frame_size:u64,receiver:FlumeReveiver<DiskMessage>)->DiskConfig{
        return DiskConfig{
            receiver:receiver,
            path:p,
            frame_size:frame_size
        };
    }
}

//-------------------------------
//map message
//-------------------------------

pub struct MapAddMessage{
    pub index:u64,
    pub value:Vec<u8>,
    pub signal:Arc<Mutex<Signal>>
}

pub struct MapGetMessage{
    pub index:u64,
    pub signal:Arc<Mutex<Signal>>,
}

pub struct MapRemoveMessage{
    pub pointer:Pointer,
    pub signal:Arc<Mutex<Signal>>
}

pub enum MapMessage{
    Get(MapGetMessage),
    Add(MapAddMessage),
    Remove(MapRemoveMessage)
}

//-------------------------------
//locator message
//-------------------------------

pub struct LocatorNext{
    pub signal:Arc<Mutex<Signal>>
}

pub struct LocatorAdd{
    pub value:Vec<u8>,
    pub signal:Arc<Mutex<Signal>>
}

pub struct LocatorReset{
    pub pointer:Pointer,
    pub signal:Arc<Mutex<Signal>>
}

pub struct LocatorRemove{
    pub pointer:Pointer,
    pub signal:Arc<Mutex<Signal>>
}

pub enum LocatorMessage{
    Next(LocatorNext),
    Add(LocatorAdd),
    Reset(LocatorReset),
    Remove(LocatorRemove)
}

//-------------------------------
//map config
//-------------------------------

pub struct MapConfig{
    // pub disk_senders:HashMap<u8,FlumeSender<DiskMessage>>,
    // pub locator_sender:FlumeSender<LocatorMessage>,

    pub map_index:u8,

    pub file_path:String,
    // pub disk_sender:FlumeSender<DiskMessage>,
    pub locator_sender:FlumeSender<LocatorMessage>,

    pub receiver:FlumeReveiver<MapMessage>,
    pub reader:Reader,
    // pub items:Vec<u64>,
    // pub items_in_processing:Vec<u64>,
    pub num_of_writers:u8,
    pub min_que_size:u64,
    pub expand_size:u64,
}

impl MapConfig{
    pub fn new(
        map_index:u8,
        file_path:String,
        // disk_sender:FlumeSender<DiskMessage>,
        reader:Reader,
        locator_sender:FlumeSender<LocatorMessage>,
        // items:Vec<u64>,
        num_of_writers:u8,
        min_que_size:u64,
        expand_size:u64,
    )->(MapConfig,FlumeSender<MapMessage>){
        let (sender, receiver) = FlumeBounded(5000);
        // let (sender, receiver) = FlumeUnBounded();
        return (
            MapConfig{
                map_index:map_index,
                file_path:file_path,
                // disk_senders:HashMap::new(),
                // locator_sender:locator_sender,
                reader:reader,
                receiver:receiver,
                locator_sender:locator_sender,
                // items:items,
                num_of_writers:num_of_writers,
                min_que_size:min_que_size,
                expand_size:expand_size
                // items_in_processing:Vec::new()
            },
            sender
        );
    }
}