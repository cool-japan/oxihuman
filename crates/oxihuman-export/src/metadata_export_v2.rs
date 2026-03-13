// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Metadata export v2: key-value metadata with typed values.
use std::collections::HashMap;
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MetaValueV2 { Text(String), Number(f64), Flag(bool) }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetadataExportV2 { pub entries: HashMap<String, MetaValueV2>, pub author: String }
#[allow(dead_code)]
pub fn new_metadata_export_v2(author:&str) -> MetadataExportV2 { MetadataExportV2 { entries:HashMap::new(), author:author.to_string() } }
#[allow(dead_code)]
pub fn meta_v2_set_text(m:&mut MetadataExportV2,key:&str,val:&str) { m.entries.insert(key.to_string(),MetaValueV2::Text(val.to_string())); }
#[allow(dead_code)]
pub fn meta_v2_set_number(m:&mut MetadataExportV2,key:&str,val:f64) { m.entries.insert(key.to_string(),MetaValueV2::Number(val)); }
#[allow(dead_code)]
pub fn meta_v2_set_flag(m:&mut MetadataExportV2,key:&str,val:bool) { m.entries.insert(key.to_string(),MetaValueV2::Flag(val)); }
#[allow(dead_code)]
pub fn meta_v2_count(m:&MetadataExportV2)->usize { m.entries.len() }
#[allow(dead_code)]
pub fn meta_v2_has_key(m:&MetadataExportV2,key:&str)->bool { m.entries.contains_key(key) }
#[allow(dead_code)]
pub fn meta_v2_to_json(m:&MetadataExportV2)->String { format!("{{\"author\":\"{}\",\"entries\":{}}}", m.author, m.entries.len()) }
#[allow(dead_code)]
pub fn meta_v2_validate(m:&MetadataExportV2)->bool { !m.author.is_empty() }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let m=new_metadata_export_v2("test");assert_eq!(m.author,"test");}
    #[test] fn test_set_text(){let mut m=new_metadata_export_v2("a");meta_v2_set_text(&mut m,"title","Hello");assert!(meta_v2_has_key(&m,"title"));}
    #[test] fn test_set_number(){let mut m=new_metadata_export_v2("a");meta_v2_set_number(&mut m,"version",1.0);assert_eq!(meta_v2_count(&m),1);}
    #[test] fn test_set_flag(){let mut m=new_metadata_export_v2("a");meta_v2_set_flag(&mut m,"visible",true);assert_eq!(meta_v2_count(&m),1);}
    #[test] fn test_count(){let m=new_metadata_export_v2("a");assert_eq!(meta_v2_count(&m),0);}
    #[test] fn test_has_key(){let m=new_metadata_export_v2("a");assert!(!meta_v2_has_key(&m,"missing"));}
    #[test] fn test_to_json(){let m=new_metadata_export_v2("author");assert!(meta_v2_to_json(&m).contains("author"));}
    #[test] fn test_validate(){let m=new_metadata_export_v2("a");assert!(meta_v2_validate(&m));}
    #[test] fn test_validate_empty(){let m=new_metadata_export_v2("");assert!(!meta_v2_validate(&m));}
    #[test] fn test_overwrite(){let mut m=new_metadata_export_v2("a");meta_v2_set_text(&mut m,"k","v1");meta_v2_set_text(&mut m,"k","v2");assert_eq!(meta_v2_count(&m),1);}
}
