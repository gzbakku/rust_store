
use std::collections::HashMap;
// use std::time::Instant;

mod unhandled;
mod key;
mod value;

#[derive(Debug,Clone)]
pub struct Write{
    pub start:usize,
    pub end:usize
}

#[derive(Debug,Clone)]
pub enum PointerType{
    Data,Corrupt,Empty
}

#[derive(Debug,Clone)]
pub struct Pointer{
    pub pointer_type:PointerType,
    pub start_on_map_cursor:usize,
    pub boundry:((usize,usize),(usize,usize)),//(map(start,end),buffer(start,end))
    pub key:((usize,usize),(usize,usize),Vec<u8>),//(map(start,end),buffer(start,end))
    pub value:((usize,usize),(usize,usize))//(map(start,end),buffer(start,end))
}

impl Pointer{
    pub fn new()->Pointer{
        Pointer{
            pointer_type:PointerType::Corrupt,
            start_on_map_cursor:0,
            boundry:((0,0),(0,0)),
            key:((0,0),(0,0),vec![]),
            value:((0,0),(0,0))
        }
    }
    pub fn clear(&mut self){
        self.pointer_type = PointerType::Empty;
        self.key = ((0,0),(0,0),vec![]);
        self.value = ((0,0),(0,0));
    }
    pub fn point(len:usize,map_cursor:usize,buffer_cursor:usize,pt:PointerType)->Pointer{
        let mut hold = Pointer::new();
        hold.start_on_map_cursor = map_cursor;
        hold.boundry.0.0 = map_cursor;
        hold.boundry.0.1 = map_cursor+len-1;
        hold.boundry.1.0 = buffer_cursor;
        hold.boundry.1.1 = buffer_cursor+len-1;
        hold.pointer_type = pt;
        return hold;
    }
    pub fn data(
        len:usize,
        map_cursor:usize,
        start:usize,end:usize,
        key_start:usize,key_end:usize,key:Vec<u8>,
        value_start:usize,value_end:usize
    )->Pointer{
        let mut hold = Pointer::new();
        //pointer
        hold.pointer_type = PointerType::Data;
        hold.start_on_map_cursor = map_cursor;
        //boundry
        hold.boundry.0.0 = map_cursor;
        hold.boundry.0.1 = map_cursor+len-1;
        hold.boundry.1.0 = start;
        hold.boundry.1.1 = end;
        //key
        let key_len = key_end - key_start;
        hold.key.0.0 = map_cursor + key_start;
        hold.key.0.1 = map_cursor + key_start + key_len;
        hold.key.1.0 = key_start;
        hold.key.1.1 = key_end;
        hold.key.2 = key;
        //value
        let value_len = value_end - value_start;
        hold.value.0.0 = map_cursor + value_start;
        hold.value.0.1 = map_cursor + value_start + value_len - 1;
        hold.value.1.0 = value_start;
        hold.value.1.1 = value_end;
        return hold;
    }
}

#[derive(Debug)]
pub enum ReaderFunc{
    Flag,Key,Value
}

#[derive(Debug)]
pub struct Reader{
    pub no_flag_in_buffer:bool,
    pub empty_map:HashMap<usize,(usize,(usize,usize))>,//<empty_index,(len,(start,end))>
    pub empty_start:HashMap<usize,usize>,//<start,empty_index>
    pub empty_end:HashMap<usize,usize>,//<end,empty_index>
    pub empty_index:usize,
    pub pointers:HashMap<Vec<u8>,((usize,usize),(usize,usize))>,//(boundry(start,end),value(start,end))
    pub corrupt:Vec<(usize,(usize,usize))>,//(len,(start,end))
    pub build_map:bool,
    pub to_find:Vec<Vec<u8>>,
    pub find_values:bool,
    pub get_values_controll:bool,
    pub values:Vec<(Vec<u8>,Vec<u8>)>,
    pub map_cursor:usize,
    pub buffer_cursor:usize,
    pub buffer:Vec<u8>,
    pub func:ReaderFunc,
    pub flag:(bool,usize,usize),//found,start,end
    pub key:(bool,u8,u64,(usize,usize),Vec<u8>),//found,byte_len,len,(start,end)
    pub value:(bool,u8,u64,(usize,usize)),//found,byte_len,len,(start,end)
    pub end:(bool,usize,usize)//foudn start,end
}

impl Reader{
    pub fn enable_find(&mut self){
        self.find_values = true;
    }
    pub fn get_values(&mut self,b:bool){
        self.get_values_controll = b;
    }
    pub fn find_key(&mut self,i:Vec<u8>){
        self.to_find.push(i);
    }
    pub fn new()->Reader{
        Reader{
            no_flag_in_buffer:false,
            pointers:HashMap::new(),
            empty_map:HashMap::new(),//<empty_index,(len,(start,end))>
            empty_start:HashMap::new(),//<start,empty_index>
            empty_end:HashMap::new(),//<end,empty_index>
            empty_index:1,
            corrupt:vec![],
            build_map:true,
            to_find:Vec::new(),
            find_values:false,
            get_values_controll:false,
            values:Vec::new(),
            map_cursor:0,
            buffer_cursor:0,
            buffer:vec![],
            func:ReaderFunc::Flag,
            flag:(false,0,0),
            key:(false,0,0,(0,0),vec![]),
            value:(false,0,0,(0,0)),
            end:(false,0,0)
        }
    }
    pub fn with_capacity(map_capacity:usize,buffer_capacity:usize)->Reader{
        Reader{
            no_flag_in_buffer:false,
            pointers:HashMap::with_capacity(map_capacity),
            empty_map:HashMap::new(),//<empty_index,(len,(start,end))>
            empty_start:HashMap::new(),//<start,empty_index>
            empty_end:HashMap::new(),//<end,empty_index>
            empty_index:1,
            corrupt:vec![],
            build_map:true,
            to_find:Vec::new(),
            find_values:false,
            get_values_controll:false,
            values:Vec::new(),
            map_cursor:0,
            buffer_cursor:0,
            buffer:Vec::with_capacity(buffer_capacity),
            func:ReaderFunc::Flag,
            flag:(false,0,0),
            key:(false,0,0,(0,0),vec![]),
            value:(false,0,0,(0,0)),
            end:(false,0,0)
        }
    }
    pub fn fill(&mut self,key:Vec<u8>,value_size:usize)->Result<Write,&'static str>{

        // let time_fill_find = Instant::now();

        //----------------------------
        //zero align boundries
        //----------------------------
        let total_len = 39+key.len()+value_size;
        let value_start_at = 36+key.len();

        //----------------------------
        //find a empty space
        //----------------------------

        let mut empty_index = 0;
        let mut empty_found = false;
        for (key,value) in self.empty_map.iter(){
            if value.0 >= total_len{
                empty_found = true;
                empty_index = *key;
                break;
            }
        }
        if !empty_found{
            return Err("full");
        }

        let empty = self.empty_map[&empty_index];
        let value_start = empty.1.0 + value_start_at;
        let value_end = empty.1.0 + 36+key.len()+value_size-1;
        let pointer_start = empty.1.0;
        let pointer_end = empty.1.0 + total_len - 1;

        //----------------------------
        //make pointer
        //----------------------------
        match self.pointers.insert(key, ((pointer_start,pointer_end),(value_start,value_end))){
            Some(_)=>{},
            None=>{}
        }

        //----------------------------
        //edit empty
        //----------------------------
        if total_len == empty.0{
            match self.empty_map.get_mut(&empty_index){
                Some(e)=>{
                    self.empty_start.remove(&e.1.0);
                    self.empty_end.remove(&e.1.1);
                },
                None=>{
                    return Err("failed-edit-empty");
                }
            }
            self.empty_map.remove(&empty_index);
        } else {
            match self.empty_map.get_mut(&empty_index){
                Some(e)=>{
                    self.empty_start.remove(&e.1.0);
                    e.1.0 = empty.1.0 + total_len;
                    e.0 = e.1.1 - e.1.0 + 1;
                    self.empty_start.insert(e.1.0,empty_index);
                },
                None=>{
                    return Err("failed-edit-empty");
                }
            }
        }

        // return Err("no_error");

        //----------------------------
        //return write
        //----------------------------
        return Ok(Write{
            start:pointer_start,
            end:pointer_end
        });

    }
    pub fn clear(&mut self,key:&Vec<u8>)->Result<(),&'static str>{

        if !self.pointers.contains_key(key){
            return Ok(());
        }

        let pointer:((usize,usize),(usize,usize));
        match self.pointers.remove(key){
            Some(v)=>{pointer = v;},
            None=>{
                return Err("failed-get-pointer");
            }
        }

        let mut previous_empty_index:usize = 0;
        let mut previous_empty_found = false;
        //if boundry start position is greater then zero
        //check if there is a empty ending at previous position
        if pointer.0.0 > 0{
            match self.empty_end.get(&(pointer.0.0 - 1)){
                Some(v)=>{
                    previous_empty_index = v.clone();
                    previous_empty_found = true;
                },
                None=>{}
            }
        }

        let mut next_empty_index:usize = 0;
        let mut next_empty_found = false;
        match self.empty_start.get(&(pointer.0.1 + 1)){
            Some(v)=>{
                next_empty_index = v.clone();
                next_empty_found = true;
            },
            None=>{}
        }

        if previous_empty_found{
            // println!("previous found");
            let mut next_empty_end = 0;
            //remove previous start pointer
            self.empty_end.remove(&self.empty_map[&previous_empty_index].1.1);
            if next_empty_found{
                next_empty_end = self.empty_map[&next_empty_index].1.1;
                //remove next emtpy boundry pointers
                self.empty_start.remove(&self.empty_map[&next_empty_index].1.0);
                self.empty_end.remove(&self.empty_map[&next_empty_index].1.1);
            }
            match self.empty_map.get_mut(&previous_empty_index){
                Some(e)=>{
                    if next_empty_found{
                        e.1.1 = next_empty_end;
                        e.0 = e.1.1 - e.1.0 + 1;
                    } else {
                        e.1.1 = pointer.0.1;
                        e.0 = e.1.1 - e.1.0 + 1;
                    }
                    self.empty_end.insert(e.1.1,previous_empty_index);
                },
                None=>{
                    return Err("failed-get-previous");
                }
            }
            if next_empty_found{
                self.empty_map.remove(&next_empty_index);
            }
        } else

        if next_empty_found{
            self.empty_start.remove(&self.empty_map[&next_empty_index].1.0);
            match self.empty_map.get_mut(&next_empty_index){
                Some(e)=>{
                    e.1.0 = pointer.0.0;
                    e.0 = e.1.1 - e.1.0 + 1;
                    self.empty_start.insert(e.1.0,next_empty_index);
                },
                None=>{
                    return Err("failed-get-previous");
                }
            }
        } else

        if !next_empty_found && !previous_empty_found{
            self.empty_start.insert(pointer.0.0,self.empty_index);
            self.empty_end.insert(pointer.0.1,self.empty_index);
            self.empty_map.insert(self.empty_index,(pointer.0.1 - pointer.0.0 + 1,pointer.0));
            self.empty_index += 1;
        } 
        
        else {
            return Err("unhandled_error");
        }

        return Ok(());

    }//clear
    pub fn find(&self,key:&Vec<u8>)->Result<((usize,usize),(usize,usize)),()>{
        match self.pointers.get(key){
            Some(v)=>{
                return Ok(*v);
            },
            None=>{
                return Err(());
            }
        }
    }
    pub fn map(&mut self,buffer:&mut Vec<u8>){

        // println!("mapping len : {:?}",buffer.len());

        loop{

            // if self.buffer.len() == 0{
            //     break;
            // }

            // let pointer_time = Instant::now();
            match read(self,buffer){
                Ok(v)=>{

                    // println!("pointer_time : {:?}",pointer_time.elapsed());

                    // println!("pointer found");

                    // println!("{:?}",buffer.len());

                    //extract pointer from buffer
                    let pointer_len = v.boundry.0.1-v.boundry.0.0+1;
                    let new_buffer = self.buffer.split_off(v.boundry.1.1+1);
                    let old_buffer = &self.buffer;
                    self.buffer_cursor = 0;
                    self.map_cursor += pointer_len;

                    //collect values
                    if self.find_values && !self.get_values_controll{
                        // println!("find values");
                        for i in self.to_find.iter(){
                            if i == &v.key.2{
                                let mut collect_value:Vec<u8> = vec![];
                                for i in v.value.1.0..v.value.1.1{
                                    collect_value.push(old_buffer[i]);
                                }
                                self.values.push((v.key.2.clone(),collect_value));
                            }
                        }
                    } else
                    if self.get_values_controll{
                        // println!("get values");
                        let mut collect_value:Vec<u8> = vec![];
                        for i in v.value.1.0..v.value.1.1{
                            collect_value.push(old_buffer[i]);
                        }
                        self.values.push((v.key.2.clone(),collect_value));
                    }

                    //build map
                    if self.build_map{
                        match v.pointer_type{
                            PointerType::Data=>{
                                match self.pointers.insert(v.key.2,(v.boundry.0,v.value.0)){
                                    Some(_)=>{},
                                    None=>{}
                                }
                            },
                            _=>{}
                        }
                    }

                    // println!("old buffer : {:?}",old_buffer.len());

                    

                    //place new buffer
                    self.buffer = new_buffer;

                    // println!("pointer_finish : {:?}",pointer_time.elapsed());

                    

                },
                Err(_e)=>{
                    break;
                }
            }
        }

        // return Ok(());

    }
    pub fn end(&mut self)->Result<(),&'static str>{

        if self.buffer.len() == 0{
            return Ok(());
        }

        loop{

            match read(self,&mut vec![]){
                Ok(v)=>{

                    //extract pointer from buffer
                    let pointer_len = v.boundry.0.1-v.boundry.0.0+1;
                    let new_buffer = self.buffer.split_off(v.boundry.1.1+1);
                    let old_buffer = &self.buffer;
                    self.buffer_cursor = 0;
                    self.map_cursor += pointer_len;

                    //collect values
                    if self.find_values && !self.get_values_controll{
                        for i in self.to_find.iter(){
                            if i == &v.key.2{
                                let mut collect_value:Vec<u8> = vec![];
                                for i in v.value.1.0..v.value.1.1{
                                    collect_value.push(old_buffer[i]);
                                }
                                self.values.push((v.key.2.clone(),collect_value));
                            }
                        }
                    } else
                    if self.get_values_controll{
                        let mut collect_value:Vec<u8> = vec![];
                        for i in v.value.1.0..v.value.1.1{
                            collect_value.push(old_buffer[i]);
                        }
                        self.values.push((v.key.2.clone(),collect_value));
                    }

                    //build map
                    if self.build_map{
                        match v.pointer_type{
                            PointerType::Data=>{
                                match self.pointers.insert(v.key.2,(v.boundry.0,v.value.0)){
                                    Some(_)=>{},
                                    None=>{}
                                }
                            },
                            _=>{}
                        }
                    }

                    //place new buffer
                    self.buffer = new_buffer;

                },
                Err(_e)=>{
                    break;
                }
            }
        }

        if self.buffer.len() > 0{
            unhandled::init(self,self.buffer.len());
        }

        return Ok(());

    }
    pub fn reset(&mut self){
        //flag
        self.flag.0 = false;
        self.flag.1 = 0;
        self.flag.2 = 0;
        //key
        self.key.0 = false;
        self.key.1 = 0;
        self.key.2 = 0;
        self.key.3 = (0,0);
        //value
        self.value.0 = false;
        self.value.1 = 0;
        self.value.2 = 0;
        self.value.3 =(0,0);
        //end
        self.end.0 = false;
        self.end.1 = 0;
        self.end.2 = 0;
    }
    pub fn flush(&mut self){
        unhandled::init(self,self.buffer.len());
        self.reset();
    }
    pub fn expand(&mut self,num_of_bytes:usize)->Result<(),&'static str>{

        //find last empty section
        let mut empty_found = false;
        let mut empty_index:usize = 0;
        match self.empty_end.get(&(self.map_cursor-1)){
            Some(v)=>{
                empty_index = *v;
                empty_found = true;
            },
            None=>{}
        }

        //update last empty section
        if empty_found{
            match self.empty_map.get_mut(&empty_index){
                Some(e)=>{
                    e.1.1 += num_of_bytes;
                    e.0 = e.1.1 - e.1.0 + 1;
                },
                None=>{
                    return Err("failed-edit-last-empty_section");
                }
            }
        } else {
            //make new empty section at the end
            self.empty_map.insert(self.empty_index,(
                num_of_bytes,
                (self.map_cursor,self.map_cursor+num_of_bytes-1)
            ));
            self.empty_end.insert(self.map_cursor+num_of_bytes-1,self.empty_index);
            self.empty_start.insert(self.map_cursor,self.empty_index);
            self.empty_index += 1;
        }

        return Ok(());

    }
}

fn read(reader:&mut Reader,buffer:&mut Vec<u8>)->Result<Pointer,&'static str>{

    if buffer.len() > 0{
        reader.buffer.append(buffer);
    }
        
    //find flag
    if reader.flag.0 == false {
        // let start_flag_time = Instant::now();
        match vector_in_vector(&reader.buffer,&vec![0,1,0],reader.buffer_cursor){
            Ok(l)=>{
                // println!("start_flag_time : {:?}",start_flag_time.elapsed());
                reader.no_flag_in_buffer = false;
                if l.0 > 0{
                    // let unhandled_time = Instant::now();
                    unhandled::init(reader,l.0);
                    // println!("unhandled_time : {:?}",unhandled_time.elapsed());
                }
                if true {
                    reader.flag.0 = true;
                    reader.flag.1 = reader.buffer_cursor;
                    reader.flag.2 = reader.buffer_cursor+2;
                    reader.buffer_cursor = 3;
                }
            },
            Err(_)=>{
                if reader.buffer.len() > 6{
                    reader.buffer_cursor = reader.buffer.len() - 5;
                } else {
                    reader.buffer_cursor = reader.buffer.len();
                }
                return Err("not_found-flag");
            }
        }
    }

    if true {
        if reader.flag.0 == true && reader.key.0 == false{
            match key::init(reader){
                Ok(_)=>{},
                Err(_)=>{
                    return Err("not_found-key");
                }
            }
        }
        if reader.flag.0 == true && reader.key.0 == true && reader.value.0 == false{
            match value::init(reader){
                Ok(_)=>{},
                Err(_)=>{
                    return Err("not_found-value");
                }
            }
        }
        if 
            reader.flag.0 == true && 
            reader.key.0 == true && 
            reader.value.0 == true &&
            reader.end.0 == true
        {
            let build = Pointer::data(
                reader.end.2-reader.flag.1+1,
                reader.map_cursor,
                reader.flag.1,reader.end.2,
                reader.key.3.0, reader.key.3.1, reader.key.4.split_off(0),
                reader.value.3.0, reader.value.3.1,
            );
            reader.reset();
            return Ok(build);
        }
    }

    return Err("no_error");

}

fn vector_in_vector(v1:&Vec<u8>,v2:&Vec<u8>,start_cursor:usize)->Result<(usize,usize),()>{

    let mut v2_cursor:usize = 0;
    let mut found_some = false;
    let mut final_found = false;
    let mut v1_index = start_cursor;
    let mut start_index = 0;
    let mut end_index = 0;

    for n in start_cursor..v1.len(){
        let i = &v1[n];
        if !found_some{
            if i == &v2[0]{
                found_some = true;
                v2_cursor = 1;
                start_index = v1_index;
            }
        } else {
            if i != &v2[v2_cursor]{
                if i == &v2[0]{
                    found_some = true;
                    v2_cursor = 1;
                    start_index = v1_index;
                } else {
                    found_some = false;
                    v2_cursor = 0;
                }
            } else {
                if v2_cursor == v2.len() - 1{
                    final_found = true;
                    end_index = v1_index;
                    break;
                } else {
                    v2_cursor += 1;
                }
            }
        }
        v1_index += 1;
    }

    if final_found{
        Ok((start_index,end_index))
    } else {
        Err(())
    }

}