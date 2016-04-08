use rustc_serialize::json;
use std::collections::HashMap;

#[derive(RustcDecodable, Debug)]
pub struct Field {
    pub index: u16,
    pub pattern: String,
}

#[derive(RustcDecodable, Debug)]
pub struct Proto {
    pub msg_type: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct Config {
    field: HashMap<String, Field>,
    proto: HashMap<String, Proto>,
    index_field: HashMap<u16, String>,
    msg_proto: HashMap<String, String>,
}

impl Field {
    pub fn new_nil() -> Field {
        Field {
            index: 0,
            pattern: "nil".to_string(),
        }
    }

    pub fn is_nil_type(&self) -> bool {
        return self.index == 0 && "nil" == self.pattern;
    }

    pub fn new(pattern: String) -> Field {
        Field {
            index: 0,
            pattern: pattern,
        }
    }
}

impl Config {
    pub fn new_empty() -> Config {
        Config {
            field: HashMap::new(),
            proto: HashMap::new(),
            index_field: HashMap::new(),
            msg_proto: HashMap::new(),
        }
    }
    pub fn new_by_map(field: HashMap<String, Field>, proto: HashMap<String, Proto>) -> Config {
        let mut index_field: HashMap<u16, String> = HashMap::new();
        let mut msg_proto: HashMap<String, String> = HashMap::new();
        for (name, f) in &field {
            index_field.insert(f.index, name.clone());
        }
        for (name, p) in &proto {
            msg_proto.insert(name.clone(), p.msg_type.clone());
        }
        Config {
            field: field,
            proto: proto,
            index_field: index_field,
            msg_proto: msg_proto,
        }
    }

    pub fn new_by_full_str(config: &str) -> Option<Config> {
        let info = json::Json::from_str(config);
        if info.is_err() {
            return None;
        }
        let info = info.ok().unwrap();
        let field = info.find("field");
        let proto = info.find("proto");
        if field.is_none() || proto.is_none() {
            return None;
        }
        let field: Result<HashMap<String, Field>, _> = json::decode(&field.unwrap().to_string());
        let proto: Result<HashMap<String, Proto>, _> = json::decode(&proto.unwrap().to_string());
        if field.is_err() || proto.is_err() {
            return None;
        }
        Some(Self::new_by_map(field.ok().unwrap(), proto.ok().unwrap()))
    }

    pub fn new(field: &str, proto: &str) -> Option<Config> {
        let field: Result<HashMap<String, Field>, _> = json::decode(field);
        let proto: Result<HashMap<String, Proto>, _> = json::decode(proto);
        if field.is_err() || proto.is_err() {
            return None;
        }
        Some(Self::new_by_map(field.ok().unwrap(), proto.ok().unwrap()))
    }

    pub fn get_field_by_name(&self, name: &String) -> Option<&Field> {
        self.field.get(name)
    }

    pub fn get_field_by_index(&self, index: &u16) -> Option<&Field> {
        let name = unwrap_or!(self.get_field_index_name(index), return None);
        self.field.get(name)
    }

    pub fn get_proto_by_name(&self, name: &String) -> Option<&Proto> {
        self.proto.get(name)
    }

    pub fn get_field_index_name(&self, index: &u16) -> Option<&String> {
        self.index_field.get(index)
    }

    pub fn get_proto_msg_type(&self, name: &String) -> Option<&String> {
        self.msg_proto.get(name)
    }
}
