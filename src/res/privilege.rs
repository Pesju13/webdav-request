use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Default, Debug, Deserialize)]
pub struct CurrentUserPrivilegeSet {
    #[serde(rename = "privilege", default)]
    privileges: Vec<PrivilegeType>,
}

impl CurrentUserPrivilegeSet {
    pub fn privilege(self) -> Privilege {
        Privilege::from(self.privileges)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeType {
    Read,
    Write,
    ReadAcl,
    WriteAcl,
    All,
    None,
}
impl<'de> Deserialize<'de> for PrivilegeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(match value {
            Value::Object(map) => {
                if map.contains_key("read") {
                    PrivilegeType::Read
                } else if map.contains_key("write") {
                    PrivilegeType::Write
                } else if map.contains_key("read_acl") {
                    PrivilegeType::ReadAcl
                } else if map.contains_key("write_acl") {
                    PrivilegeType::WriteAcl
                } else if map.contains_key("all") {
                    PrivilegeType::All
                } else {
                    PrivilegeType::None
                }
            }
            _ => PrivilegeType::None,
        })
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default)]
pub struct Privilege {
    read: bool,
    write: bool,
    read_acl: bool,
    write_acl: bool,
    all: bool,
}

impl Privilege {
    pub fn read(&self) -> bool {
        self.read
    }
    pub fn write(&self) -> bool {
        self.write
    }
    pub fn read_acl(&self) -> bool {
        self.read_acl
    }
    pub fn write_acl(&self) -> bool {
        self.write_acl
    }
    pub fn all(&self) -> bool {
        self.all
    }
}

impl From<Vec<PrivilegeType>> for Privilege {
    fn from(value: Vec<PrivilegeType>) -> Self {
        Self {
            read: value.contains(&PrivilegeType::Read),
            write: value.contains(&PrivilegeType::Write),
            read_acl: value.contains(&PrivilegeType::ReadAcl),
            write_acl: value.contains(&PrivilegeType::WriteAcl),
            all: value.contains(&PrivilegeType::All),
        }
    }
}
