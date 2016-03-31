# td_rp
tickbh rust bin protocol

[![Build Status](https://api.travis-ci.org/tickbh/td_proto_rust.svg?branch=master)](https://travis-ci.org/tickbh/td_proto_rust)

## suport type
base type is contain "u8",   "i8",   "u16",   "i16",   "u32",   "i32",   "float",   "string",   "raw",   "map"

array type is contain "u8[]", "i8[]", "u16[]", "i16[]", "u32[]", "i32[]", "float[]", "string[]", "raw[]", "map[]"

# data detail
data will be format like as Id, Type, Data store by little endian, Id is 2bytes, Type is 2bytes
 - "u8",   "i8",   "u16",   "i16" -- 2bytes 
 - "u32",  "i32",  "float"        -- 4bytes, float decode with i32 div 1000
 - "string",  "raw"               -- 2bytes len, len bytes datas
 - map                            -- key always encode string, contains id, type, value is base value, end will key type is nil
 - array                          -- write base data, stop with id = 0, type = 0

# example data u8
```rust
extern crate td_proto_rust;
use td_rp::{Value, Config, Buffer};

fn test_head_field(buffer : &mut Buffer, index : u16, t : u16) {
    // first index bytes
    let data: &mut [u8; 2] = &mut [0, 0];
    let size = buffer.read(data).unwrap();
    assert_eq!(size, 2);
    let val = u16::from_le(unsafe { mem::transmute::<[u8;2], u16>(*data) });
    assert_eq!(val, index);

    // first type bytes
    let size = buffer.read(data).unwrap();
    assert_eq!(size, 2);

    let val = u16::from_le(unsafe { mem::transmute::<[u8;2], u16>(*data) });
    assert_eq!(val, t);
}

fn test_encode_u8() {
    let config = Config::new_empty();
    let mut buffer = Buffer::new();
    let value = Value::from(1 as u8);
    td_rp::encode_field(&mut buffer, &config, &value).unwrap();
    td_rp::encode_field(&mut buffer, &config, &value).unwrap();

    // first read field
    test_head_field(&mut buffer, 0, td_rp::TYPE_U8);
    // after index type is data
    let data: &mut [u8; 2] = &mut [0, 0];
    let size = buffer.read(data).unwrap();
    assert_eq!(size, 2);
    assert_eq!(data[0], 1);
    assert_eq!(data[1], 0);

    // second read field
    let read = td_rp::decode_field(&mut buffer, &config).unwrap();
    match read {
        Value::U8(val) => assert_eq!(val, 1),
        _ => unreachable!("it will not read"),
    }

    let size = buffer.read(data).unwrap();
    assert_eq!(size, 0);
}
```
>the bytes is  
>[0, 0, 1, 0, 1, 0] -- [0, 0] is id = 0, [1, 0] is type = 1 is TYPE_U8, [1, 0] is data is 1u8

# example proto
```rust
extern crate td_proto_rust;
use td_rp::{Value, Config, Buffer};

fn test_base_proto() {
    let config = td_rp::Config::new(" { \"name\" : { \"index\" :    1, \"pattern\" : \"string\" }, \
                                        \"index\" : { \"index\" :    2, \"pattern\" : \"u16\" },  \
                                        \"sub_name\" : { \"index\" :    3, \"pattern\" :\"string\" }   }",
        "{\"cmd_test_op\"        : { \"index\" :    1, \"args\" : [ \"map\" ] }}");
    let config = config.unwrap();
    let mut hash_value = HashMap::<String, Value>::new();
    hash_value.insert("name".to_string(), Value::from("I'm a chinese people".to_string()));
    hash_value.insert("sub_name".to_string(), Value::from("tickdream".to_string()));
    hash_value.insert("index".to_string(), Value::from(1 as u16));

    {
        let mut buffer = td_rp::encode_proto(&config, &"cmd_test_op".to_string(), vec![Value::from(hash_value.clone())]).unwrap();
        // just read field
        let read = td_rp::decode_proto(&mut buffer, &config).unwrap();
        match read {
            (name, val) => {
                assert_eq!(name, "cmd_test_op".to_string());
                assert_eq!(val[0], Value::from(hash_value));
            }
        }
    }
}
```
it will encode Vec<Value> accords to proto name like as "cmd_test_op" define args is [map]

# compatible
it will ensure data decoded maximum
 - old protocol can decode the new protocol if new protocol not change the old field info, but it will miss some info
 - new protocol can decode the old protocol all datas
