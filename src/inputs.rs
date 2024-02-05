use derived_deref::Deref;

#[derive(Clone)]
pub enum InputData {
    Insert(InsertData),
    Remove(RemoveData),
    Update(UpdateData),
    ReadInput(String),
    Invalid,
    ClearTerm,
    NewLine,
}

#[derive(Clone)]
pub struct InsertData {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Deref)]
pub struct RemoveData {
    pub key: String,
}

#[derive(Clone)]
pub struct UpdateData {
    pub key: String,
    pub updated_value: String,
}

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
        if val.contains("read") {
            return Self::ReadInput(val);
        }
        if val.contains("insert") {
            return Self::Insert(InsertData::from(val));
        }
        if val.contains("remove") {
            return Self::Remove(RemoveData::from(val));
        }
        if val.contains("update") {
            return Self::Update(UpdateData::from(val));
        }
        if val.contains("clear") {
            return Self::ClearTerm;
        }
        return Self::NewLine;
    }
}
