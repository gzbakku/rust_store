use gzb_binary_69::Reader;
// use flume::unbounded as FlumeUnBounded;
use flume::bounded as FlumeBounded;
use flume::Receiver as FlumeReveiver;
use flume::Sender as FlumeSender;
use crate::workers::{Signal,SignalData};
// use crate::workers::Debugger;
use std::sync::Arc;
use tokio::sync::{Notify,Mutex};

pub struct Config{
    pub path:String,
    pub frame:u64,
    pub disk_writers:u64
}

impl Config{
    pub fn new(path:String,frame_size:u64,num_of_writers:u64)->Config{
        Config{
            path:path,
            frame:frame_size,
            disk_writers:num_of_writers
        }
    }
}

pub struct DiskGetMessage{
    pub index:u64,
    pub value_boundry:(usize,usize),//(start,end)
    pub signal:Arc<Mutex<SignalData>>,
    pub notify:Arc<Notify>
}   

pub struct DiskAddMessage{
    // pub debugger:Arc<Mutex<Debugger>>,
    pub start_at:u64,
    pub value:Vec<u8>,
    pub signal:Arc<Mutex<Signal>>,
    pub notify:Arc<Notify>
}

pub struct DiskRemoveMessage{
    pub boundry:(usize,usize),//(start,end)
    pub signal:Arc<Mutex<Signal>>,
    pub notify:Arc<Notify>
}

pub enum DiskMessage{
    Expand((Arc<Mutex<Signal>>,Arc<Notify>)),
    Add(DiskAddMessage),
    AddUn((u64,Vec<u8>)),
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
    pub fn new(p:String,frame_size:u64)->(DiskConfig,FlumeSender<DiskMessage>){
        let (sender,receiver) = FlumeBounded(5000);
        // let (sender, receiver) = FlumeUnBounded();
        return (
            DiskConfig{
                receiver:receiver,
                path:p,
                frame_size:frame_size
            },
            sender
        );
    }
}

pub struct MapAddMessage{
    pub value:Vec<u8>,
    pub signal:Arc<Mutex<Signal>>,
    pub notify:Arc<Notify>
}

pub struct MapGetMessage{
    pub signal:Arc<Mutex<SignalData>>,
    pub notify:Arc<Notify>
}

pub struct MapRemoveMessage{
    pub index:u64,
    pub signal:Arc<Mutex<Signal>>,
    pub notify:Arc<Notify>
}

pub enum MapMessage{
    Get(MapGetMessage),
    Add(MapAddMessage),
    AddUn(Vec<u8>),
    Remove(MapRemoveMessage)
}

pub struct MapConfig{
    pub disk_sender:FlumeSender<DiskMessage>,
    pub reader:Reader,
    pub receiver:FlumeReveiver<MapMessage>,
    pub items:Vec<u64>,
    pub items_in_processing:Vec<u64>,
    pub frame_size:u64
}

impl MapConfig{
    pub fn new(r:Reader,disk_sender:FlumeSender<DiskMessage>,items:Vec<u64>,frame_size:u64)->(MapConfig,FlumeSender<MapMessage>){
        let (sender, receiver) = FlumeBounded(5000);
        // let (sender, receiver) = FlumeUnBounded();
        return (
            MapConfig{
                disk_sender:disk_sender,
                reader:r,
                receiver:receiver,
                items:items,
                frame_size:frame_size,
                items_in_processing:Vec::new()
            },
            sender
        );
    }
}