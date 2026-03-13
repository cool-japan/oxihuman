// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Query {
    pub name: String,
    pub params: std::collections::HashMap<String, String>,
}

pub struct QueryResult {
    pub name: String,
    pub data: String,
    pub success: bool,
}

pub fn new_query(name: &str) -> Query {
    Query {
        name: name.to_string(),
        params: std::collections::HashMap::new(),
    }
}

pub fn query_set_param(q: &mut Query, key: &str, val: &str) {
    q.params.insert(key.to_string(), val.to_string());
}

pub fn query_get_param<'a>(q: &'a Query, key: &str) -> Option<&'a str> {
    q.params.get(key).map(|s| s.as_str())
}

pub fn new_query_result(name: &str, data: &str, success: bool) -> QueryResult {
    QueryResult {
        name: name.to_string(),
        data: data.to_string(),
        success,
    }
}

pub fn query_result_is_success(r: &QueryResult) -> bool {
    r.success
}

pub fn query_result_data(r: &QueryResult) -> &str {
    &r.data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_query() {
        /* create query with name */
        let q = new_query("GetUser");
        assert_eq!(q.name, "GetUser");
        assert!(q.params.is_empty());
    }

    #[test]
    fn test_query_set_get_param() {
        /* set and get query params */
        let mut q = new_query("Find");
        query_set_param(&mut q, "id", "42");
        assert_eq!(query_get_param(&q, "id"), Some("42"));
        assert_eq!(query_get_param(&q, "missing"), None);
    }

    #[test]
    fn test_new_query_result() {
        /* create query result */
        let r = new_query_result("GetUser", "data", true);
        assert!(query_result_is_success(&r));
        assert_eq!(query_result_data(&r), "data");
    }

    #[test]
    fn test_query_result_failure() {
        /* failed result has success=false */
        let r = new_query_result("Find", "error", false);
        assert!(!query_result_is_success(&r));
    }

    #[test]
    fn test_query_multiple_params() {
        /* multiple params stored correctly */
        let mut q = new_query("Search");
        query_set_param(&mut q, "a", "1");
        query_set_param(&mut q, "b", "2");
        assert_eq!(query_get_param(&q, "a"), Some("1"));
        assert_eq!(query_get_param(&q, "b"), Some("2"));
    }

    #[test]
    fn test_query_result_data() {
        /* data field returned correctly */
        let r = new_query_result("Q", "payload", true);
        assert_eq!(query_result_data(&r), "payload");
    }
}
