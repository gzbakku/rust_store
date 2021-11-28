

use crate::reader::{Reader};
use crate::workers::{u64_from_bytes,p_error};

const ERROR:bool = true;

pub fn init(reader:&mut Reader)->Result<(),()>{

    if reader.value.1 == 0{
        if reader.buffer.len() < 17+reader.key.1 as usize+reader.key.2 as usize{
            // p_error("short-value_len-buff_size-value",ERROR);
            return Err(());
        }
        if
            reader.buffer[reader.buffer_cursor+1] != 0 ||
            reader.buffer[reader.buffer_cursor+2] != 5 ||
            reader.buffer[reader.buffer_cursor+3] != 0 
        {
            p_error("failed-parse-buffer_len-value_buffer-value",ERROR);
            reader.flush();
            return Err(());
        }
        reader.value.1 = reader.buffer[reader.buffer_cursor];
        reader.buffer_cursor += 4;
    }

    if reader.value.2 == 0{
        if reader.buffer.len() < 21+reader.key.1 as usize+reader.key.2 as usize + reader.value.1 as usize{
            // p_error("short-len-value-buff_size-value",ERROR);
            return Err(());
        }
        let mut collect_value_len_buffer = vec![];
        for n in reader.buffer_cursor..reader.buffer_cursor+reader.value.1 as usize{
            collect_value_len_buffer.push(reader.buffer[n]);
        }
        match u64_from_bytes(collect_value_len_buffer){
            Ok(v)=>{
                reader.value.2 = v;
                reader.buffer_cursor += reader.value.1 as usize;
            },
            Err(_)=>{
                p_error("failed-parse-buffer_len-value_buffer-value",ERROR);
                reader.flush();
                return Err(());
            }
        }
    }

    if reader.value.0 == false{
        if 
            reader.buffer.len() < 
            24+
            reader.key.1 as usize+
            reader.key.2 as usize + 
            reader.value.1 as usize + 
            reader.value.2 as usize - 1 
        {
            // p_error("short-value_len-buff_size-value",ERROR);
            return Err(());
        }
        if
            reader.buffer[reader.buffer_cursor+0] != 0 ||
            reader.buffer[reader.buffer_cursor+1] != 6 ||
            reader.buffer[reader.buffer_cursor+2] != 0 
        {
            p_error("invalid-flag-value_buffer-start-value",ERROR);
            reader.flush();
            return Err(());
        }
        reader.buffer_cursor += 3;
        if
            reader.buffer[reader.buffer_cursor+0+reader.value.2 as usize] != 0 ||
            reader.buffer[reader.buffer_cursor+1+reader.value.2 as usize] != 7 ||
            reader.buffer[reader.buffer_cursor+2+reader.value.2 as usize] != 0 
        {
            p_error("invalid-flag-value_buffer-end-value",ERROR);
            reader.flush();
            return Err(());
        }
        reader.value.0 = true;
        reader.value.3.0 = reader.buffer_cursor;
        reader.value.3.1 = reader.buffer_cursor + reader.value.2 as usize;
        reader.end.0 = true;
        reader.end.1 = reader.buffer_cursor+0+reader.value.2 as usize;
        reader.end.2 = reader.buffer_cursor+2+reader.value.2 as usize;
        reader.buffer_cursor += 2 + reader.value.2 as usize;
        // println!("{:?}",reader.end);
    }

    return Ok(());

}

/*

0, 1, 0,                        a   3
8,                              b   1
0, 2, 0,                        c   3   //7
0, 0, 0, 0, 0, 0, 0, 3,         d   b=  //7+8              //15
0, 3, 0,                        e   3   //7+b+3            //10+b           //18
107, 101, 121,                  f   d=+p                   //10+b+d         //18+d
0, 4, 0,                        g   3   //10+b+d+3         //13+b+d         //21+d
8,                              h   1                      //12+b+d+3       //22+d
0, 5, 0,                        i   3   //13+b+d+1+3       //17+b+d         //25+d
0, 0, 0, 0, 0, 0, 0, 5,         j   h=                                      //33+d            
0, 6, 0,                        k   3   //17+b+d+h+3       //21+b+d+h       //36+d
118, 97, 108, 117, 101,         l   j=+p                                    //36+d+j
0, 7, 0                         n   3   //21+b+d+h+j+3     //24+b+d+h+j     //49+d+j

//totla overheade
24+b+d+h+j = 18+f+18+l+3

*/