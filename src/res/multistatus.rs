use super::privilege::CurrentUserPrivilegeSet;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MultiStatus {
    #[serde(default)]
    pub response: Vec<DResponse>,
}

impl MultiStatus {
    pub fn parse(s: &str) -> Result<Self, quick_xml::DeError> {
        quick_xml::de::from_str(s)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DResponse {
    #[serde(rename = "href")]
    pub href: String,
    #[serde(rename = "propstat")]
    pub prop_stat: PropStat,
}

impl DResponse {
    pub fn upwrap(self) -> (String, Prop) {
        (self.href, self.prop_stat.prop)
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct PropStat {
    pub prop: Prop,
    pub status: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Prop {
    #[serde(rename = "displayname")]
    pub display_name: String,
    #[serde(default, rename = "getcontenttype")]
    pub content_type: String,
    #[serde(default, rename = "getlastmodified")]
    pub last_modified: String,
    #[serde(rename = "getcontentlength", default)]
    pub content_length: u64,
    #[serde(alias = "iscollection", default)]
    pub collection: bool,
    #[serde(rename = "resourcetype", default)]
    pub resource_type: Option<ResourceType>,
    #[serde(rename = "current-user-privilege-set", default)]
    pub current_user_privilege_set: Option<CurrentUserPrivilegeSet>,
    pub creationdate: Option<String>,
    pub supportedlock: Option<SupportedLock>,
}

impl Prop {
    pub fn is_collection(&self) -> bool {
        self.collection
            || self
                .resource_type
                .as_ref()
                .is_some_and(|ty| ty.collection.is_some())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupportedLock {
    pub lockentry: Option<LockEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LockEntry {
    pub lockscope: Option<LockScope>,
    pub locktype: Option<LockType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LockScope {
    pub exclusive: Option<()>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LockType {
    pub write: Option<()>,
}
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ResourceType {
    collection: Option<()>,
}
