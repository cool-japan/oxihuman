// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! HATEOAS link builder export stub — generates hypermedia-as-the-engine-of-application-state
//! link collections for mesh/avatar REST responses.

/// HATEOAS link relation type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkRel {
    Self_,
    Next,
    Prev,
    Related,
    Described,
    Custom(String),
}

impl LinkRel {
    /// Relation name string.
    pub fn as_str(&self) -> &str {
        match self {
            LinkRel::Self_ => "self",
            LinkRel::Next => "next",
            LinkRel::Prev => "prev",
            LinkRel::Related => "related",
            LinkRel::Described => "describedby",
            LinkRel::Custom(s) => s.as_str(),
        }
    }
}

/// A single HATEOAS link.
#[derive(Debug, Clone)]
pub struct HateoasLink {
    pub rel: String,
    pub href: String,
    pub method: Option<String>,
    pub title: Option<String>,
}

/// A HATEOAS resource representation.
#[derive(Debug, Clone, Default)]
pub struct HateoasResource {
    pub links: Vec<HateoasLink>,
    pub resource_type: String,
    pub resource_id: String,
}

/// A HATEOAS export session.
#[derive(Debug, Default)]
pub struct HateoasExport {
    pub resources: Vec<HateoasResource>,
    pub base_url: String,
}

/// Create a new HATEOAS export session.
pub fn new_hateoas_export(base_url: &str) -> HateoasExport {
    HateoasExport {
        resources: Vec::new(),
        base_url: base_url.to_owned(),
    }
}

/// Add a resource to the export.
pub fn add_hateoas_resource(export: &mut HateoasExport, resource_type: &str, resource_id: &str) {
    export.resources.push(HateoasResource {
        links: Vec::new(),
        resource_type: resource_type.to_owned(),
        resource_id: resource_id.to_owned(),
    });
}

/// Add a link to the last resource.
pub fn add_hateoas_link(
    export: &mut HateoasExport,
    rel: LinkRel,
    href: &str,
    method: Option<&str>,
) {
    if let Some(res) = export.resources.last_mut() {
        res.links.push(HateoasLink {
            rel: rel.as_str().to_owned(),
            href: href.to_owned(),
            method: method.map(str::to_owned),
            title: None,
        });
    }
}

/// Set a title on the last link of the last resource.
pub fn set_link_title(export: &mut HateoasExport, title: &str) {
    if let Some(res) = export.resources.last_mut() {
        if let Some(link) = res.links.last_mut() {
            link.title = Some(title.to_owned());
        }
    }
}

/// Number of resources.
pub fn hateoas_resource_count(export: &HateoasExport) -> usize {
    export.resources.len()
}

/// Total number of links across all resources.
pub fn total_hateoas_links(export: &HateoasExport) -> usize {
    export.resources.iter().map(|r| r.links.len()).sum()
}

/// Count links with a specific relation across all resources.
pub fn links_with_rel(export: &HateoasExport, rel: &str) -> usize {
    export
        .resources
        .iter()
        .flat_map(|r| r.links.iter())
        .filter(|l| l.rel == rel)
        .count()
}

/// Find a resource by type and id.
pub fn find_hateoas_resource<'a>(
    export: &'a HateoasExport,
    resource_type: &str,
    resource_id: &str,
) -> Option<&'a HateoasResource> {
    export
        .resources
        .iter()
        .find(|r| r.resource_type == resource_type && r.resource_id == resource_id)
}

/// Render a resource's links to a JSON-style string.
pub fn render_hateoas_links(resource: &HateoasResource) -> String {
    let links: Vec<String> = resource
        .links
        .iter()
        .map(|l| format!(r#"{{"rel":"{}","href":"{}"}}"#, l.rel, l.href))
        .collect();
    format!("[{}]", links.join(","))
}

/// Serialize metadata to JSON-style string.
pub fn hateoas_export_to_json(export: &HateoasExport) -> String {
    format!(
        r#"{{"base_url":"{}","resource_count":{},"total_links":{}}}"#,
        export.base_url,
        hateoas_resource_count(export),
        total_hateoas_links(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        /* fresh export has no resources */
        let e = new_hateoas_export("https://api.example.com");
        assert_eq!(hateoas_resource_count(&e), 0);
    }

    #[test]
    fn add_resource_increments_count() {
        /* adding a resource increases count */
        let mut e = new_hateoas_export("https://api.example.com");
        add_hateoas_resource(&mut e, "mesh", "1");
        assert_eq!(hateoas_resource_count(&e), 1);
    }

    #[test]
    fn add_link_stored() {
        /* link is stored on last resource */
        let mut e = new_hateoas_export("https://api.example.com");
        add_hateoas_resource(&mut e, "mesh", "1");
        add_hateoas_link(&mut e, LinkRel::Self_, "/mesh/1", Some("GET"));
        assert_eq!(e.resources[0].links.len(), 1);
    }

    #[test]
    fn total_links_counted() {
        /* total links across all resources */
        let mut e = new_hateoas_export("https://api.example.com");
        add_hateoas_resource(&mut e, "mesh", "1");
        add_hateoas_link(&mut e, LinkRel::Self_, "/mesh/1", Some("GET"));
        add_hateoas_link(&mut e, LinkRel::Next, "/mesh/2", None);
        assert_eq!(total_hateoas_links(&e), 2);
    }

    #[test]
    fn links_with_rel_self_counted() {
        /* self links counted correctly */
        let mut e = new_hateoas_export("https://api.example.com");
        add_hateoas_resource(&mut e, "mesh", "1");
        add_hateoas_link(&mut e, LinkRel::Self_, "/mesh/1", None);
        add_hateoas_link(&mut e, LinkRel::Related, "/bones", None);
        assert_eq!(links_with_rel(&e, "self"), 1);
    }

    #[test]
    fn find_resource_success() {
        /* find returns matching resource */
        let mut e = new_hateoas_export("https://api.example.com");
        add_hateoas_resource(&mut e, "avatar", "42");
        assert!(find_hateoas_resource(&e, "avatar", "42").is_some());
    }

    #[test]
    fn find_resource_missing_none() {
        /* missing resource returns None */
        let e = new_hateoas_export("https://api.example.com");
        assert!(find_hateoas_resource(&e, "ghost", "99").is_none());
    }

    #[test]
    fn render_links_contains_rel() {
        /* rendered links contain rel field */
        let mut e = new_hateoas_export("https://api.example.com");
        add_hateoas_resource(&mut e, "mesh", "1");
        add_hateoas_link(&mut e, LinkRel::Self_, "/mesh/1", None);
        let rendered = render_hateoas_links(&e.resources[0]);
        assert!(rendered.contains("\"self\""));
    }

    #[test]
    fn link_rel_custom_name() {
        /* custom rel name is preserved */
        let rel = LinkRel::Custom("edit".to_owned());
        assert_eq!(rel.as_str(), "edit");
    }

    #[test]
    fn json_contains_base_url() {
        /* JSON includes base URL */
        let e = new_hateoas_export("https://mesh.api.com");
        assert!(hateoas_export_to_json(&e).contains("mesh.api.com"));
    }
}
