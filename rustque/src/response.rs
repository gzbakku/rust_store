use std::sync::Arc;
use tokio::sync::Mutex;
use crate::workers::{Signal,SignalData,Pointer};
use tokio::sync::Notify;

pub struct QueResponse{
    signal:Arc<Mutex<Signal>>,
    sleeper:Arc<Notify>
}

impl QueResponse{
    pub fn new(signal:Arc<Mutex<Signal>>,sleeper:Arc<Notify>)->QueResponse{
        QueResponse{
            signal:signal,
            sleeper:sleeper
        }
    }
    pub async fn check(&mut self)->bool{
        self.sleeper.notified().await;
        if !Signal::check(&self.signal).await{
            return false;
        } else {
            return true;
        }
    }
    pub async fn data(self)->Option<(Vec<u8>,Pointer)>{
        match Signal::get(self.signal).await{
            SignalData::Value(v)=>{
                Some(v)
            },
            _=>{None}
        }
    }
}