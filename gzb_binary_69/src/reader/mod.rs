
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
    pub boundry:((usize,usize),(usize,usize)),//(map(start,end),buffer(start,end))
    pub key:((usize,usize),(usize,usize),Vec<u8>),//(map(start,end),buffer(start,end))
    pub value:((usize,usize),(usize,usize))//(map(start,end),buffer(start,end))
}

impl Pointer{
    pub fn new()->Pointer{
        Pointer{
            pointer_type:PointerType::Corrupt,
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
        hold.boundry.0.0 = map_cursor;
        hold.boundry.0.1 = map_cursor+len-1;
        hold.boundry.1.0 = buffer_cursor;
        hold.boundry.1.1 = buffer_cursor+len-1;
        hold.pointer_type = pt;
        return hold;
    }
    pub fn data(
        map_cursor:usize,
        start:usize,end:usize,
        key_start:usize,key_end:usize,key:Vec<u8>,
        value_start:usize,value_end:usize
    )->Pointer{

        let mut hold = Pointer::new();
        //pointer
        hold.pointer_type = PointerType::Data;
        //boundry
        hold.boundry.0.0 = start + map_cursor;
        hold.boundry.0.1 = end + map_cursor;
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
        hold.value.0.1 = map_cursor + value_start + value_len;
        hold.value.1.0 = value_start;
        hold.value.1.1 = value_end;

        // println!("");

        return hold;

    }
}

#[derive(Debug)]
pub enum ReaderFunc{
    Flag,Key,Value
}

// #[derive(Debug,Default)]
// pub struct Calc{
//     pub total_empty:u128,
//     pub count_time:u128,
//     pub empty_end_pointers_time:u128,
//     pub update_reader_time:u128,
//     pub empty_countinues_time:u128,
//     pub previous_empty_found_time:u128,
//     pub previous_empty_not_found_time:u128,   
//     pub total_empty_time:u128,
//     pub unhandled_empty_time:u128,
// }

#[derive(Debug)]
pub struct Reader{
    // pub anchor_time:Instant,
    // pub calc:Calc,
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
            // anchor_time:Instant::now(),
            // calc:Calc::default(),
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
            // anchor_time:Instant::now(),
            // calc:Calc::default(),
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

        loop{

            match read(self,buffer){
                Ok(v)=>{

                    //collect values
                    if self.find_values && !self.get_values_controll{
                        for i in self.to_find.iter(){
                            if i == &v.key.2{
                                let mut collect_value:Vec<u8> = vec![];
                                for i in v.value.1.0..=v.value.1.1{
                                    collect_value.push(self.buffer[i]);
                                }
                                self.values.push((v.key.2.clone(),collect_value));
                            }
                        }
                    } else
                    if self.get_values_controll{
                        let mut collect_value:Vec<u8> = vec![];
                        for i in v.value.1.0..=v.value.1.1{
                            collect_value.push(self.buffer[i]);
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

                    if self.dump(){self.buffer_cursor += 1;} else {break;}

                },
                Err(_e)=>{
                    break;
                }
            }
        }

    }
    pub fn end(&mut self)->Result<(),&'static str>{

        if self.buffer.len() == 0{
            return Ok(());
        }

        loop{

            match read(self,&mut vec![]){
                Ok(v)=>{

                    //collect values
                    if self.find_values && !self.get_values_controll{
                        for i in self.to_find.iter(){
                            if i == &v.key.2{
                                let mut collect_value:Vec<u8> = vec![];
                                for i in v.value.1.0..=v.value.1.1{
                                    collect_value.push(self.buffer[i]);
                                }
                                self.values.push((v.key.2.clone(),collect_value));
                            }
                        }
                    } else
                    if self.get_values_controll{
                        let mut collect_value:Vec<u8> = vec![];
                        for i in v.value.1.0..=v.value.1.1{
                            collect_value.push(self.buffer[i]);
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

                    if self.dump(){self.buffer_cursor += 1;} else {break;}

                },
                Err(_e)=>{
                    break;
                }
            }
        }

        if self.buffer.len() > 0{
            unhandled::init(self,self.buffer_cursor,self.buffer.len()-1);
        }
        self.buffer.clear();

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
        // println!("flushing");
        unhandled::init(self,self.flag.1,self.flag.2);
        self.reset();
    }
    pub fn expand(&mut self,num_of_bytes:usize)->Result<(),&'static str>{

        //find last empty section
        let mut empty_found = false;
        let mut empty_index:usize = 0;
        if self.map_cursor > 0{
            match self.empty_end.get(&(self.map_cursor-1)){
                Some(v)=>{
                    empty_index = *v;
                    empty_found = true;
                },
                None=>{}
            }
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
    pub fn dump(&mut self)->bool{
        if self.buffer.len() == 0{
            self.map_cursor += self.buffer_cursor;
            // println!("++++++++ map_cursor expanded 0");
            self.buffer_cursor = 0;
            return false;
        }
        if self.flag.0{return true;}
        if self.buffer_cursor+1 >= self.buffer.len(){
            // self.buffer.clear();
            // self.map_cursor += self.buffer_cursor+1;
            // println!("++++++++ map_cursor expanded 1");
            // self.buffer_cursor = 0;
            return false;
        }
        if !self.flag.0{
            if self.buffer_cursor > 100_000_000{
                self.buffer = self.buffer.split_off(self.buffer_cursor);
                self.map_cursor += self.buffer_cursor;
                // println!("++++++++ map_cursor expanded 2");
                self.buffer_cursor = 0;
            }
        }
        if self.buffer.len() > 0{
            return true;
        } else {
            return false;
        }
    }
    
}

fn read(reader:&mut Reader,buffer:&mut Vec<u8>)->Result<Pointer,&'static str>{
    
    if buffer.len() > 0{
        reader.buffer.append(buffer);
    }

    if false{
        crate::workers::buff_print(&reader.buffer);
    }

    if !reader.flag.0{
        // reader.calc.total_empty += 1;
        // let unhandled_empty_time = Instant::now();
        unhandled::empty_counter(reader);
        // reader.calc.unhandled_empty_time += unhandled_empty_time.elapsed().as_nanos();
    }

    if !reader.dump(){
        return Err("no_buffer");
    }
        
    //find flag
    if reader.flag.0 == false {
        let base_buffer_cursor = reader.buffer_cursor;
        match vector_in_vector(&reader.buffer,&vec![0,1,0],reader.buffer_cursor){
            Ok(l)=>{
                reader.no_flag_in_buffer = false;
                if l.0 > base_buffer_cursor{
                    unhandled::init(reader,base_buffer_cursor,l.0-1);
                }
                if true {
                    reader.flag.0 = true;
                    reader.flag.1 = l.0;
                    reader.flag.2 = l.1;
                    reader.buffer_cursor += 3;
                }
            },
            Err(_)=>{
                return Err("not_found-flag");
            }
        }
    }

    //find key & value
    if true {
        // let part_process_time = Instant::now();
        if reader.flag.0 == true && reader.key.0 == false{
            match key::init(reader){
                Ok(_)=>{
                    // println!("part_key : {:?}",part_process_time.elapsed());
                },
                Err(_)=>{
                    return Err("not_found-key");
                }
            }
        }
        if reader.flag.0 == true && reader.key.0 == true && reader.value.0 == false{
            match value::init(reader){
                Ok(_)=>{
                    // println!("part_value : {:?}",part_process_time.elapsed());
                },
                Err(_)=>{
                    return Err("not_found-value");
                }
            }
        }
    }

    //check if a valid pointer is found
    if true{
        if
            reader.flag.0 == true && 
            reader.key.0 == true && 
            reader.value.0 == true &&
            reader.end.0 == true
        {
            let build = Pointer::data(
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

// pub fn calc(&mut self){

//     // pub total_empty:u128,
//     // pub count_time:u128,
//     // pub empty_end_pointers_time:u128,
//     // pub update_reader_time:u128,
//     // pub empty_countinues_time:u128,
//     // pub previous_empty_found_time:u128,
//     // pub previous_empty_not_found_time:u128,   
//     // pub total_empty_time:u128,
//     // pub unhandled_empty_time:u128,

//     println!("\n===========================================\n");

//     let count_time = (self.calc.count_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);
//     let empty_end_pointers_time = (self.calc.empty_end_pointers_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);
//     let update_reader_time = (self.calc.update_reader_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);
//     let empty_countinues_time = (self.calc.empty_countinues_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);
//     let previous_empty_found_time = (self.calc.previous_empty_found_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);
//     let previous_empty_not_found_time = (self.calc.previous_empty_not_found_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);
//     let total_empty_time = (self.calc.total_empty_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);
//     let unhandled_empty_time = (self.calc.unhandled_empty_time as f64) / (self.calc.total_empty as f64) / (1_000_000.0);

//     println!("total_empty : {:?}",self.calc.total_empty);
//     println!("calc.previous_empty_found_time : {:?}",self.calc.previous_empty_found_time);
//     println!("count_time : {:?}",count_time);
//     println!("empty_end_pointers_time : {:?}",empty_end_pointers_time);
//     println!("update_reader_time : {:?}",update_reader_time);
//     println!("empty_countinues_time : {:?}",empty_countinues_time);
//     println!("previous_empty_found_time : {:?}",previous_empty_found_time);
//     println!("previous_empty_not_found_time : {:?}",previous_empty_not_found_time);
//     println!("total_empty_time : {:?}",total_empty_time);
//     println!("unhandled_empty_time : {:?}",unhandled_empty_time);

//     let mut build_bar_line = String::new();
//     build_bar_line += &format!("{},",crate::workers::concat_str(count_time.to_string()));
//     build_bar_line += &format!(" {},",crate::workers::concat_str(empty_end_pointers_time.to_string()));
//     build_bar_line += &format!(" {},",crate::workers::concat_str(update_reader_time.to_string()));
//     build_bar_line += &format!(" {},",crate::workers::concat_str(empty_countinues_time.to_string()));
//     build_bar_line += &format!(" {},",crate::workers::concat_str(previous_empty_found_time.to_string()));
//     build_bar_line += &format!(" {},",crate::workers::concat_str(previous_empty_not_found_time.to_string()));
//     build_bar_line += &format!(" {},",crate::workers::concat_str(total_empty_time.to_string()));
//     build_bar_line += &format!(" {}",crate::workers::concat_str(unhandled_empty_time.to_string()));

//     println!("\n{}\n",build_bar_line);

//     println!("\n===========================================\n");

// }