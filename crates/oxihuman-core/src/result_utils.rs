// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub fn ok_or_else_str<T>(opt: Option<T>, msg: &str) -> Result<T, String> {
    opt.ok_or_else(|| msg.to_string())
}

pub fn map_err_str<T, E: std::fmt::Display>(r: Result<T, E>) -> Result<T, String> {
    r.map_err(|e| e.to_string())
}

pub fn collect_results<T>(results: Vec<Result<T, String>>) -> Result<Vec<T>, Vec<String>> {
    let mut oks = Vec::new();
    let mut errs = Vec::new();
    for r in results {
        match r {
            Ok(v) => oks.push(v),
            Err(e) => errs.push(e),
        }
    }
    if errs.is_empty() {
        Ok(oks)
    } else {
        Err(errs)
    }
}

pub fn first_ok<T>(results: impl IntoIterator<Item = Result<T, String>>) -> Option<T> {
    results.into_iter().find_map(|r| r.ok())
}

pub fn transpose_option_result<T>(opt: Option<Result<T, String>>) -> Result<Option<T>, String> {
    match opt {
        None => Ok(None),
        Some(Ok(v)) => Ok(Some(v)),
        Some(Err(e)) => Err(e),
    }
}

pub fn unwrap_or_default_str(opt: Option<String>) -> String {
    opt.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_or_else_str() {
        /* Some becomes Ok, None becomes Err */
        assert_eq!(ok_or_else_str(Some(42), "oops"), Ok(42));
        assert_eq!(ok_or_else_str::<i32>(None, "oops"), Err("oops".to_string()));
    }

    #[test]
    fn test_map_err_str() {
        /* map error to string */
        let r: Result<i32, i32> = Err(42);
        assert_eq!(map_err_str(r), Err("42".to_string()));
    }

    #[test]
    fn test_collect_results_all_ok() {
        /* all ok gives Ok(vec) */
        let v = vec![Ok(1), Ok(2), Ok(3)];
        assert_eq!(collect_results(v), Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_collect_results_has_err() {
        /* any error gives Err(errs) */
        let v: Vec<Result<i32, String>> = vec![Ok(1), Err("bad".into()), Ok(3)];
        assert!(collect_results(v).is_err());
    }

    #[test]
    fn test_first_ok() {
        /* returns first Ok value */
        let v: Vec<Result<i32, String>> = vec![Err("a".into()), Ok(2), Ok(3)];
        assert_eq!(first_ok(v), Some(2));
    }

    #[test]
    fn test_transpose_option_result() {
        /* transposes Option<Result> to Result<Option> */
        assert_eq!(transpose_option_result::<i32>(None), Ok(None));
        assert_eq!(transpose_option_result(Some(Ok(5))), Ok(Some(5)));
        assert!(transpose_option_result::<i32>(Some(Err("e".into()))).is_err());
    }

    #[test]
    fn test_unwrap_or_default_str() {
        /* None gives empty string */
        assert_eq!(unwrap_or_default_str(None), "");
        assert_eq!(unwrap_or_default_str(Some("hi".to_string())), "hi");
    }
}
