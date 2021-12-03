use std::io::{Cursor};
use byteorder::{BigEndian, WriteBytesExt,ReadBytesExt};
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub fn debug_error(e:&'static str,c:bool){
    if c{
        println!("!!! {}",e);
    }
}

#[allow(dead_code)]
pub fn debug_message(e:&'static str,c:bool){
    if c{
        println!("!!! {}",e);
    }
}

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
        let lock = hold.lock().await;
        return lock.result;
    }
}

pub struct SignalData{
    pub result:bool,
    pub data:Vec<u8>,
    pub index:u64
}

impl SignalData{
    pub fn new()->Arc<Mutex<SignalData>>{
        Arc::new(
            Mutex::new(
                SignalData{
                    result:false,
                    data:Vec::new(),
                    index:0
                }
            )
        )
    }
    pub async fn ok(hold:Arc<Mutex<SignalData>>,data:Vec<u8>,index:u64){
        let mut lock = hold.lock().await;
        lock.data = data;
        lock.result = true;
        lock.index = index;
    }
}

// #[derive(Debug)]
// pub struct Debugger{
//     updates:Vec<&'static str>,
//     errors:Vec<&'static str>
// }

// impl Debugger{
//     pub fn new()->Arc<Mutex<Debugger>>{
//         Arc::new(
//             Mutex::new(
//                 Debugger{
//                     updates:vec![],
//                     errors:vec![]
//                 }
//             )
//         )
//     }
//     pub async fn update(hold:&Arc<Mutex<Debugger>>,m:&'static str){
//         let mut lock = hold.lock().await;
//         lock.updates.push(m);
//     }
//     pub async fn error(hold:&Arc<Mutex<Debugger>>,m:&'static str){
//         let mut lock = hold.lock().await;
//         lock.errors.push(m);
//     }
//     pub async fn print(hold:&Arc<Mutex<Debugger>>){
//         let lock = hold.lock().await;
//         println!("{:#?}",lock);
//     }
// }