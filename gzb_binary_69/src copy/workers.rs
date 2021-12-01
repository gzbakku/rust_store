

use byteorder::{BigEndian, WriteBytesExt,ReadBytesExt};
use std::io::{Cursor};
use crate::{Reader,PointerType};

#[allow(dead_code)]
pub fn u64_to_be(u:u64)->Vec<u8>{
    u.to_le_bytes().to_vec()
}

#[allow(dead_code)]
pub fn u64_from_bytes(pool:Vec<u8>)->Result<u64,()>{
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

#[allow(dead_code)]
pub fn p_error(e:&'static str,p:bool){
    if p {
        println!("!!! {}",e);
    }
}

#[allow(dead_code)]
pub fn buff_print(pool:&Vec<u8>){
    let mut index = 0;
    println!("\n=====================\n");
    for i in pool{
        print!(" | {} : {}",index,i);
        index += 1;
    }
    println!("\n=====================\n");
}

#[allow(dead_code)]
pub fn print_pointers(r:&mut Reader){
    println!("\n===");
    for i in r.map.iter(){
        println!("{:?} {:?} {:?}",i.pointer_type,i.boundry.0,u64_from_bytes(i.key.2.clone()));
    }
    println!("===\n");
}

pub fn print_last_100_pointers(r:&mut Reader){
    if r.map.len() < 100{
        return print_raw_pointers(r);
    }
    let start = r.map.len() - 101;
    let end = r.map.len() - 1;
    for n in start..=end{
        println!("{:?} {:?} {:?}",n,r.map[n].pointer_type,u64_from_bytes(r.map[n].key.2.clone()));
    }
}

pub fn print_raw_pointers(r:&mut Reader){
    println!("\n");
    let mut index = 0;
    for i in r.map.iter(){
        match i.pointer_type{
            PointerType::Corrupt=>{
                println!("{:?} {:?}",index,i);
            },
            _=>{
                println!("{:?} {:?} {:?}",index,i.pointer_type,u64_from_bytes(i.key.2.clone()));
            }
        }
        index += 1;
    }
    println!("\n");
}