use crate::multistatus::MultiStatus;

/// Represents an item within a WebDAV collection.
///
/// DavItem contains information about a file or directory in a WebDAV server,
#[derive(Default, Debug, Clone)]
pub struct DavItem {
    pub is_dir: bool,
    pub url: String,
    /// The display name of the item.
    pub name: String,
    /// The last modified timestamp of the item.
    pub last_modified: String,
    /// The size of the item in bytes.
    pub size: u64,
    /// The content type (MIME type) of the item.
    pub content_type: String,
}

impl DavItem {
    pub(crate) fn parse(multi_status: MultiStatus, url: &str) -> crate::Result<Vec<Self>> {
        let responses = multi_status.response;
        if responses.is_empty() {
            return Ok(Vec::new());
        }
        let mut buf = Vec::new();
        for item in responses.into_iter().skip(1) {
            let prop = item.prop_stat.prop;
            let item_url = format!(
                "{url}{}{}",
                if url.ends_with("/") { "" } else { "/" },
                prop.display_name
            );

            let path = percent_encoding::percent_decode_str(&item_url)
                .decode_utf8()?
                .to_string();
            buf.push(DavItem {
                is_dir: prop.is_collection(),
                url: path,
                name: prop.display_name,
                last_modified: prop.last_modified,
                size: prop.content_length,
                content_type: prop.content_type,
            });
        }

        Ok(buf)
    }
}
