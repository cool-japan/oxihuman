// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! JWT encode/decode stub — header.payload.signature (base64url, no crypto).

/// Supported JWT algorithms (stub).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JwtAlgorithm {
    HS256,
    RS256,
    None,
}

/// JWT header fields.
#[derive(Clone, Debug)]
pub struct JwtHeader {
    pub alg: JwtAlgorithm,
    pub typ: String,
}

/// JWT claims payload.
#[derive(Clone, Debug)]
pub struct JwtClaims {
    pub sub: String,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub exp: Option<u64>,
    pub iat: Option<u64>,
    pub jti: Option<String>,
}

/// A decoded JWT.
#[derive(Clone, Debug)]
pub struct DecodedJwt {
    pub header: JwtHeader,
    pub claims: JwtClaims,
    pub signature: String,
}

/// Simple stub base64url encoding (not spec-compliant, for stub use only).
pub fn base64url_encode(input: &str) -> String {
    let encoded = input
        .bytes()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    encoded.replace('=', "")
}

/// Encodes a JWT (stub — no real signing).
pub fn jwt_encode(header: &JwtHeader, claims: &JwtClaims, _secret: &str) -> String {
    let alg_str = match header.alg {
        JwtAlgorithm::HS256 => "HS256",
        JwtAlgorithm::RS256 => "RS256",
        JwtAlgorithm::None => "none",
    };
    let h = base64url_encode(&format!(
        r#"{{"alg":"{}","typ":"{}"}}"#,
        alg_str, header.typ
    ));
    let p = base64url_encode(&format!(
        r#"{{"sub":"{}","iss":"{}"}}"#,
        claims.sub,
        claims.iss.as_deref().unwrap_or("")
    ));
    let sig = base64url_encode(&format!("sig_{}", claims.sub));
    format!("{}.{}.{}", h, p, sig)
}

/// Decodes a JWT string into its parts (stub — no signature verification).
pub fn jwt_decode(token: &str) -> Result<DecodedJwt, String> {
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return Err("invalid JWT format".into());
    }
    Ok(DecodedJwt {
        header: JwtHeader {
            alg: JwtAlgorithm::HS256,
            typ: "JWT".into(),
        },
        claims: JwtClaims {
            sub: format!("stub_sub_from_{}", parts[1].len()),
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            jti: None,
        },
        signature: parts[2].to_owned(),
    })
}

/// Returns true if the JWT token is structurally valid (has 3 dot-separated parts).
pub fn jwt_is_structurally_valid(token: &str) -> bool {
    token.splitn(4, '.').count() == 3
}

/// Returns the algorithm string from a header.
pub fn algorithm_name(alg: JwtAlgorithm) -> &'static str {
    match alg {
        JwtAlgorithm::HS256 => "HS256",
        JwtAlgorithm::RS256 => "RS256",
        JwtAlgorithm::None => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_produces_three_parts() {
        let header = JwtHeader {
            alg: JwtAlgorithm::HS256,
            typ: "JWT".into(),
        };
        let claims = JwtClaims {
            sub: "user1".into(),
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            jti: None,
        };
        let tok = jwt_encode(&header, &claims, "secret");
        assert_eq!(tok.split('.').count(), 3 /* three-part JWT */);
    }

    #[test]
    fn test_decode_valid_token() {
        let header = JwtHeader {
            alg: JwtAlgorithm::HS256,
            typ: "JWT".into(),
        };
        let claims = JwtClaims {
            sub: "user2".into(),
            iss: Some("test".into()),
            aud: None,
            exp: Some(9999),
            iat: None,
            jti: None,
        };
        let tok = jwt_encode(&header, &claims, "s");
        let decoded = jwt_decode(&tok).expect("should succeed");
        assert!(!decoded.signature.is_empty());
    }

    #[test]
    fn test_decode_invalid_token_returns_error() {
        assert!(jwt_decode("bad_token").is_err());
    }

    #[test]
    fn test_structural_validity_check() {
        assert!(jwt_is_structurally_valid("a.b.c"));
        assert!(!jwt_is_structurally_valid("a.b"));
    }

    #[test]
    fn test_algorithm_name_hs256() {
        assert_eq!(algorithm_name(JwtAlgorithm::HS256), "HS256");
    }

    #[test]
    fn test_algorithm_name_none() {
        assert_eq!(algorithm_name(JwtAlgorithm::None), "none");
    }

    #[test]
    fn test_base64url_encode_not_empty() {
        let out = base64url_encode("hello");
        assert!(!out.is_empty());
    }

    #[test]
    fn test_encode_with_no_algorithm() {
        let header = JwtHeader {
            alg: JwtAlgorithm::None,
            typ: "JWT".into(),
        };
        let claims = JwtClaims {
            sub: "anon".into(),
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            jti: None,
        };
        let tok = jwt_encode(&header, &claims, "");
        assert!(tok.contains('.'));
    }

    #[test]
    fn test_decode_preserves_signature_part() {
        let tok = "aaa.bbb.ccc";
        let decoded = jwt_decode(tok).expect("should succeed");
        assert_eq!(decoded.signature, "ccc");
    }
}
