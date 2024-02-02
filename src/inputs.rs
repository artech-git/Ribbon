
#[derive(Clone)]
pub enum InputData {
    insert(InsertData),
    remove(RemoveData),
    update(UpdateData),
    Invalid,
    ClearTerm,
    NewLine
}

unsafe impl std::marker::Send for InputData { }
unsafe impl std::marker::Sync for InputData { }

#[derive(Clone)]
pub struct InsertData {
    pub key: String,
    pub value: String,
}
unsafe impl std::marker::Send for InsertData { }

#[derive(Clone)]
pub struct RemoveData {
    pub key: String,
}

unsafe impl std::marker::Send for RemoveData { }

#[derive(Clone)]    
pub struct UpdateData {
    pub key: String,
    pub updated_value: String,
}

unsafe impl std::marker::Send for UpdateData { }


impl std::convert::From<String> for InsertData {
    fn from(val: String) -> Self {
        let values = val
            .trim()
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        Self {
            key: values[1].clone(),
            value: values[2].clone(),
        }
    }
}

impl std::convert::From<String> for RemoveData {
    fn from(val: String) -> Self {
        let values = val
            .trim()
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        Self {
            key: values[1].clone(),
        }
    }
}

impl std::convert::From<String> for UpdateData {
    fn from(val: String) -> Self {
        let values = val
            .trim()
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        Self {
            key: values[1].clone(),
            updated_value: values[2].clone(),
        }
    }
}

impl std::convert::From<String> for InputData {
    fn from(val: String) -> Self {
        if val.contains(r#"\n"#){
            // println!("true value");
            return Self::NewLine; 
        }
        if val.contains("insert") {
            return Self::insert(InsertData::from(val));
        }
        if val.contains("remove") {
            return Self::remove(RemoveData::from(val));
        }
        if val.contains("update") {
            return Self::update(UpdateData::from(val));
        }
        if val.contains("clear") {
            return Self::ClearTerm;
        }
        
        return Self::Invalid;
    }
}
