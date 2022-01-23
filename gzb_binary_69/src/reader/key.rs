

//pub key:(bool,usize,usize,u8,u64,Vec<u8>),//found,start,end,byte_len,len,value,

use crate::workers::{u64_from_bytes,p_error};
use crate::reader::{Reader};

const ERROR:bool = false;

pub fn init(reader:&mut Reader)->Result<(),()>{

    if reader.key.1 == 0{
        if reader.buffer.len() - reader.flag.1 < 3+1{
            // p_error("short-flag_len-key",ERROR);
            return Err(());
        }
        reader.key.1 = reader.buffer[reader.buffer_cursor];
        reader.buffer_cursor += 1;
    }

    if reader.key.2 ==0{
        if reader.buffer.len() - reader.flag.1 < 3+1+3+reader.key.1 as usize{
            // p_error("short-key_len-buff_size-key",ERROR);
            return Err(());
        }
        if
            reader.buffer[reader.buffer_cursor+0] != 0 ||
            reader.buffer[reader.buffer_cursor+1] != 2 ||
            reader.buffer[reader.buffer_cursor+2] != 0 
        {
            // crate::workers::print_range(&reader.buffer, 0, 100);
            p_error("invalid-flag-key_len-buff_size-key",ERROR);
            reader.flush();
            return Err(());
        }
        reader.buffer_cursor += 3;
        let mut collect_len_bytes:Vec<u8> = vec![];
        for n in reader.buffer_cursor..reader.buffer_cursor+reader.key.1 as usize{
            collect_len_bytes.push(reader.buffer[n]);
        }
        match u64_from_bytes(collect_len_bytes){
            Ok(v)=>{
                reader.key.2 = v;
                reader.buffer_cursor = reader.buffer_cursor+reader.key.1 as usize;
            },
            Err(_)=>{
                p_error("invalid-buff-key_len-key",ERROR);
                reader.flush();
                return Err(());
            }
        }
    }

    if reader.key.0 == false{
        if reader.buffer.len() - reader.flag.1 < 13+reader.key.1 as usize+reader.key.2 as usize{
            // p_error("short-key_buff-key",ERROR);
            return Err(());
        }
        if
            reader.buffer[reader.buffer_cursor+0] != 0 ||
            reader.buffer[reader.buffer_cursor+1] != 3 ||
            reader.buffer[reader.buffer_cursor+2] != 0 
        {
            p_error("invalid-flag-key_buff-start-key",ERROR);
            reader.flush();
            return Err(());
        }
        //set key boundry in buffer
        reader.key.3.0 = reader.buffer_cursor+3;
        reader.key.3.1 = reader.buffer_cursor + 3 + (reader.key.2 as usize) - 1;
        reader.buffer_cursor += 3;
        reader.buffer_cursor += reader.key.2 as usize;
        if
            reader.buffer[reader.buffer_cursor+0] != 0 ||
            reader.buffer[reader.buffer_cursor+1] != 4 ||
            reader.buffer[reader.buffer_cursor+2] != 0 
        {
            p_error("invalid-flag-key_buff-end-key",ERROR);
            reader.flush();
            return Err(());
        }
        //set final key params
        reader.key.0 = true;
        reader.buffer_cursor += 3;
        let mut collect_key = vec![];
        for i in reader.key.3.0..=reader.key.3.1{
            collect_key.push(reader.buffer[i]);
        }
        reader.key.4 = collect_key;
    }

    // println!("key");

    return Ok(());

}

/*

0, 1, 0,                        a   3
8,                              b   1
0, 2, 0,                        c   3   //7
0, 0, 0, 0, 0, 0, 0, 3,         d   b=                     
0, 3, 0,                        e   3   //7+b+3            //10+b
107, 101, 121,                  f   d=+p                   //10+b+d
0, 4, 0,                        g   3   //10+b+d+3         //13+b+d
8,                              h   1
0, 5, 0,                        i   3   //13+b+d+1+3       //17+b+d
0, 0, 0, 0, 0, 0, 0, 5,         j   h=
0, 6, 0,                        k   3   //17+b+d+h+3       //21+b+d+h
118, 97, 108, 117, 101,         l   j=+p
0, 7, 0                         n   3   //21+b+d+h+j+3     //24+b+d+h+j

*/