use percent_encoding::percent_decode_str;
use reqwest::IntoUrl;
use url::Url;

use super::{multistatus::MultiStatus, privilege::Privilege, SupportedLock};

#[derive(Default, Debug)]
/// Represents a collection (directory) in a WebDAV server.
///
/// Contains information about the collection itself and its children items.
pub struct DavCollection {
    /// The full encoded href path as returned by the server.
    pub href: String,
    /// The relative path of the item from a base URL, if available.
    pub relative_href: Option<String>,
    /// The URL of the collection.
    pub url: Option<String>,
    /// The display name of the collection.
    pub display_name: String,
    /// The children items (files and directories) contained in this collection.
    pub children: Vec<DavItem>,
}
impl Iterator for DavCollection {
    type Item = DavItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.children.pop()
    }
}

impl DavCollection {
    pub fn new(status: MultiStatus) -> crate::Result<Self> {
        DavCollectionBuilder::default().status(status).build()
    }

    pub fn builder() -> DavCollectionBuilder {
        DavCollectionBuilder::default()
    }

    /// Sets the URL of this collection and calculates URLs for all children.
    ///
    /// This method updates the URL of the collection and generates URLs for all child items
    /// by combining the collection URL with each child's name.
    pub fn set_url(&mut self, url: impl IntoUrl) -> crate::Result<()> {
        let mut url = percent_decode_str(url.into_url()?.as_str())
            .decode_utf8()?
            .to_string();
        if url.ends_with('/') {
            url.pop();
        }
        for item in self.iter_mut() {
            item.url = Some(format!("{}/{}", url, item.name));
        }
        self.url = Some(url);
        Ok(())
    }

    /// Sets the relative href paths for this collection and its children.
    ///
    /// This method calculates the relative paths by removing the base URL prefix from
    /// the absolute URL. It requires that the collection's URL has been set and
    /// starts with the provided base URL.
    pub fn set_relative_href(&mut self, base_url: impl IntoUrl) -> crate::Result<()> {
        let base_url = percent_decode_str(base_url.into_url()?.as_str())
            .decode_utf8()?
            .to_string();
        let suffix = match &self.url {
            Some(url) => url
                .strip_prefix(&base_url)
                .ok_or_else(|| {
                    crate::Error::InvalidData("The `url` must start with `base_url`".into())
                })?
                .to_string(),
            None => return Err(crate::Error::InvalidData("`url` is None.".to_owned())),
        };
        let suffix = if suffix.is_empty() {
            "/".to_owned()
        } else {
            suffix
        };
        self.relative_href = Some(suffix.clone());
        let parent = if suffix.eq("/") {
            String::new()
        } else {
            suffix
        };
        for item in &mut self.children {
            item.relative_href = Some(format!("{}/{}", parent, item.name));
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &DavItem> {
        self.children.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut DavItem> {
        self.children.iter_mut()
    }
}

/// Builder for constructing a DavCollection with customizable options.
///
/// This builder provides a fluent interface to set various properties
/// needed to construct a properly configured DavCollection.
#[derive(Default)]
pub struct DavCollectionBuilder {
    url: Option<reqwest::Result<Url>>,
    base_url: Option<reqwest::Result<Url>>,
    status: Option<MultiStatus>,
}

impl DavCollectionBuilder {
    pub fn status(mut self, status: MultiStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn url(mut self, url: impl IntoUrl) -> Self {
        self.url = Some(url.into_url());
        self
    }

    pub fn base_url(mut self, base_url: impl IntoUrl) -> Self {
        self.base_url = Some(base_url.into_url());
        self
    }

    /// Sets both the URL and base URL in a single call.
    ///
    /// This is a convenience method that combines setting both the URL and base URL,
    /// which is a common pattern when you want both absolute and relative paths.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder itself for method chaining.
    pub fn with_urls(mut self, url: impl IntoUrl, base_url: impl IntoUrl) -> Self {
        self.url = Some(url.into_url());
        self.base_url = Some(base_url.into_url());
        self
    }
    pub fn build(self) -> crate::Result<DavCollection> {
        let status = self
            .status
            .ok_or_else(|| crate::Error::InvalidData("Missing `MultiStatus`.".to_string()))?;
        let mut iter = status.response.into_iter();
        let collection = iter.next().ok_or_else(|| crate::Error::InvalidResponse)?;
        let href = percent_decode_str(&collection.href)
            .decode_utf8()?
            .to_string();

        let url = match self.url {
            Some(url) => {
                let mut url = percent_decode_str(url?.as_str()).decode_utf8()?.to_string();
                if url.ends_with('/') {
                    url.pop();
                }
                Some(url)
            }
            None => None,
        };

        let suffix = if let Some(base_url) = self.base_url {
            let base_url = percent_decode_str(base_url?.as_str())
                .decode_utf8()?
                .to_string();
            let suffix = match &url {
                Some(url) => url
                    .strip_prefix(&base_url)
                    .ok_or_else(|| {
                        crate::Error::InvalidData("The `url` must start with `base_url`".into())
                    })?
                    .to_string(),
                None => return Err(crate::Error::InvalidData("`url` is None.".to_owned())),
            };
            Some(if suffix.is_empty() {
                "/".to_owned()
            } else {
                suffix
            })
        } else {
            None
        };

        let parent = suffix
            .as_ref()
            .map(|s| if s.eq("/") { String::new() } else { s.clone() });
        Ok(DavCollection {
            href,
            relative_href: suffix,
            url: url.clone(),
            display_name: collection.prop_stat.prop.display_name,
            children: iter
                .map(|node| {
                    let (href, prop) = node.upwrap();

                    DavItem {
                        is_dir: prop.is_collection(),
                        href: percent_decode_str(&href)
                            .decode_utf8()
                            .map(|s| s.to_string())
                            .unwrap_or(href.clone()),
                        relative_href: parent
                            .as_ref()
                            .map(|h| format!("{}/{}", h, prop.display_name)),
                        url: url
                            .as_ref()
                            .map(|url| format!("{url}/{}", prop.display_name)),
                        name: prop.display_name,
                        modified: prop.last_modified,
                        size: prop.content_length,
                        content_type: prop.content_type,
                        supported_lock: prop.supportedlock,
                        privilege: prop.current_user_privilege_set.map(|p| p.privilege()),
                    }
                })
                .collect(),
        })
    }
}

/// Represents an item within a WebDAV collection.
///
/// DavItem contains information about a file or directory in a WebDAV server,
/// including its path, name, size, and permissions.
#[derive(Default, Debug, Clone)]
pub struct DavItem {
    /// Whether this item is a directory (true) or file (false).
    pub is_dir: bool,
    /// The full encoded href path as returned by the server.
    pub href: String,
    /// The relative path of the item from a base URL, if available.
    pub relative_href: Option<String>,
    /// The URL of the item.
    pub url: Option<String>,
    /// The display name of the item.
    pub name: String,
    /// The last modified timestamp of the item.
    pub modified: String,
    /// The size of the item in bytes.
    pub size: u64,
    /// The content type (MIME type) of the item.
    pub content_type: String,
    /// The access privileges associated with this item.
    pub privilege: Option<Privilege>,
    pub supported_lock: Option<SupportedLock>,
}
