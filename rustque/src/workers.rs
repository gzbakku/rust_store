use std::io::{Cursor};
use byteorder::{BigEndian, WriteBytesExt,ReadBytesExt};
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub fn u64_from_bytes(pool:&Vec<u8>)->Result<u64,()>{
    // println!("h+ : {:?}",pool);
    let mut rdr = Cursor::new(pool);
    match rdr.read_u64::<BigEndian>(){
        Ok(v)=>{return Ok(v)},
        Err(_e)=>{
            // println!("_e : {:?}",_e);
            return Err(());
        }
    }
}

#[allow(dead_code)]
pub fn u64_to_bytes(n:u64)->Result<Vec<u8>,()>{
    let mut value_len_as_bytes = Vec::new();
    match value_len_as_bytes.write_u64::<BigEndian>(n){
        Ok(_)=>{
            // println!("h- : {:?}",value_len_as_bytes);
            return Ok(value_len_as_bytes);
        },
        Err(_)=>{
            return Err(());
        }
    }
}



// #[derive()]
pub struct Signal{
    result:bool
}

impl Signal{
    pub fn new()->Arc<Mutex<Signal>>{
        Arc::new(
            Mutex::new(
                Signal{
                    result:false 
                }
            )
        )
    }
    pub async fn ok(hold:Arc<Mutex<Signal>>){
        let mut lock = hold.lock().await;
        lock.result = true;
    }
    pub async fn check(hold:Arc<Mutex<Signal>>)->bool{
        let mut lock = hold.lock().await;
        return lock.result;
    }
}