use std::io::{Write};
use std::mem;

use Config;
use Value;
use Buffer;
use Field;
use RpResult;
use ErrorKind;
use {get_type_by_name, get_name_by_type, get_array_contains_type, get_type_by_value};

use STR_TYPE_NIL;

static FILL_UP : [u8;16] = [0; 16];

fn write_str_field(buffer : &mut Buffer, pattern : &str) -> RpResult<bool> {
    try!(encode_number(buffer, &Value::from(0 as u16)));
    try!(encode_number(buffer, &Value::U16(get_type_by_name(pattern))));
    Ok(true)
}

fn append_and_align(buffer : &mut Buffer, val : &[u8]) -> RpResult<()>  {
    let _add = match val.len() % 2 {
        0 => 0,
        val => 2 - val,
    };
    try!(buffer.write(val));
    // buffer.write(&FILL_UP[0..add]);
    Ok(())
}

pub fn encode_number(buffer : &mut Buffer, value : &Value) -> RpResult<()>  {
    match *value {
        Value::U8(val) => {
            try!(buffer.write(unsafe { &mem::transmute::<u8, [u8;1]>(val) }));
            try!(buffer.write(&FILL_UP[0..1]));
        }
        Value::I8(val) => {
            try!(buffer.write(unsafe { &mem::transmute::<i8, [u8;1]>(val) }));
            try!(buffer.write(&FILL_UP[0..1]));
        }
        Value::U16(val) => {
            try!(buffer.write(unsafe { &mem::transmute::<u16, [u8;2]>(val.to_le()) }));
        }
        Value::I16(val) => {
            try!(buffer.write(unsafe { &mem::transmute::<i16, [u8;2]>(val.to_le()) }));
        }
        Value::U32(val) => {
            try!(buffer.write(unsafe { &mem::transmute::<u32, [u8;4]>(val.to_le()) }));
        }
        Value::I32(val) => {
            try!(buffer.write(unsafe { &mem::transmute::<i32, [u8;4]>(val.to_le()) }));
        }
        Value::Float(val) => {
            let val = (val * 1000.0) as i32;
            try!(buffer.write(unsafe { &mem::transmute::<i32, [u8;4]>(val.to_le()) }));
        }
        _ => unreachable!("encode_number only")
    }
    Ok(())
}

pub fn encode_str_raw(buffer : &mut Buffer, value : &Value) -> RpResult<()>  {
    match *value {
        Value::Str(ref val) => {
            try!(encode_number(buffer, &Value::U16(val.len() as u16)));
            try!(append_and_align(buffer, &val.as_bytes()[..]));
        }
        Value::Raw(ref val) => {
            try!(encode_number(buffer, &Value::U16(val.len() as u16)));
            try!(append_and_align(buffer, &val[..]));
        }
        _ => unreachable!("encode_number only")
    }
    Ok(())
}

pub fn encode_map(buffer : &mut Buffer, config : &Config, value : &Value) -> RpResult<()>  {
    match *value {
        Value::Map(ref val) => {
            for (name, sub_value) in val {
                if try!(write_field(buffer, config.get_field_by_name(name))) {
                    try!(encode_field(buffer, config, sub_value));    
                }
            }
            try!(write_str_field(buffer, STR_TYPE_NIL));
        }
        _ => unreachable!("encode_number only")
    }
    Ok(())
}


pub fn write_field(buffer : &mut Buffer, field : Option<&Field>) -> RpResult<bool> {
    if field.is_none() {
        return Ok(false);
    }
    let field = field.unwrap();
    try!(encode_number(buffer, &Value::U16(field.index)));
    try!(encode_number(buffer, &Value::U16(get_type_by_name(&field.pattern))));
    Ok(true)
}

pub fn encode_field(buffer : &mut Buffer, config : &Config, value : &Value) -> RpResult<()> {
    try!(write_str_field(buffer, get_name_by_type(get_type_by_value(value))));
    match *value {
        Value::U8(_) | Value::I8(_) | Value::U16(_) | Value::I16(_) | Value::U32(_) | Value::I32(_) | Value::Float(_)  => {
            try!(encode_number(buffer, value));
        }
        Value::Str(_) | Value::Raw(_) => {
            try!(encode_str_raw(buffer, value));
        }
        Value::Map(_) => {
            try!(encode_map(buffer, config, value));
        }
        Value::Nil => {
        }
        Value::AU8(ref val) | Value::AI8(ref val) | Value::AU16(ref val) | Value::AI16(ref val) | 
        Value::AU32(ref val) | Value::AI32(ref val) | Value::AFloat(ref val) | Value::AStr(ref val) |
        Value::ARaw(ref val) | Value::AMap(ref val) => {
            let must_type = get_array_contains_type(value);
            for v in val {
                check_vailed!(v, must_type);
                try!(encode_field(buffer, config, v));
            }
            try!(write_str_field(buffer, STR_TYPE_NIL));
        }
    }
    Ok(())
}

pub fn encode_proto(buffer : &mut Buffer, config : &Config, name : &String, infos : Vec<Value>) -> RpResult<()> {
    let proto = config.get_proto_by_name(name);
    ensure!(proto.is_some(), (ErrorKind::MissingError, "missing the name protocol"));
    let proto = proto.unwrap();
    let name = config.get_proto_index_name(&proto.index);
    ensure!(name.is_some(), (ErrorKind::MissingError, "miss the name data"));
    ensure!(proto.args.len() == infos.len(), (ErrorKind::TypeNotMatchError, "the data num not match protocol args num"));
    let name = name.map(|s| s.clone());
    try!(encode_str_raw(buffer, &Value::Str(name.unwrap())));
    for info in &infos {
        //TODO match args
        try!(encode_field(buffer, config, info));
    }
    try!(write_str_field(buffer, STR_TYPE_NIL));
    Ok(())
}

