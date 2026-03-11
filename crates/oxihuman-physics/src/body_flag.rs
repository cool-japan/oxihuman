#![allow(dead_code)]

const FLAG_STATIC: u32 = 1 << 0;
const FLAG_KINEMATIC: u32 = 1 << 1;
const FLAG_DYNAMIC: u32 = 1 << 2;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BodyFlags(u32);

#[allow(dead_code)]
pub fn new_body_flags() -> BodyFlags {
    BodyFlags(0)
}

#[allow(dead_code)]
pub fn set_flag(bf: &mut BodyFlags, flag: u32) {
    bf.0 |= flag;
}

#[allow(dead_code)]
pub fn clear_flag(bf: &mut BodyFlags, flag: u32) {
    bf.0 &= !flag;
}

#[allow(dead_code)]
pub fn has_flag(bf: &BodyFlags, flag: u32) -> bool {
    (bf.0 & flag) != 0
}

#[allow(dead_code)]
pub fn is_static(bf: &BodyFlags) -> bool {
    has_flag(bf, FLAG_STATIC)
}

#[allow(dead_code)]
pub fn is_kinematic(bf: &BodyFlags) -> bool {
    has_flag(bf, FLAG_KINEMATIC)
}

#[allow(dead_code)]
pub fn is_dynamic(bf: &BodyFlags) -> bool {
    has_flag(bf, FLAG_DYNAMIC)
}

#[allow(dead_code)]
pub fn flags_to_u32(bf: &BodyFlags) -> u32 {
    bf.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let f = new_body_flags();
        assert_eq!(flags_to_u32(&f), 0);
    }

    #[test]
    fn test_set_flag() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_STATIC);
        assert!(is_static(&f));
    }

    #[test]
    fn test_clear_flag() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_KINEMATIC);
        clear_flag(&mut f, FLAG_KINEMATIC);
        assert!(!is_kinematic(&f));
    }

    #[test]
    fn test_has_flag() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_DYNAMIC);
        assert!(has_flag(&f, FLAG_DYNAMIC));
        assert!(!has_flag(&f, FLAG_STATIC));
    }

    #[test]
    fn test_is_static() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_STATIC);
        assert!(is_static(&f));
        assert!(!is_dynamic(&f));
    }

    #[test]
    fn test_is_kinematic() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_KINEMATIC);
        assert!(is_kinematic(&f));
    }

    #[test]
    fn test_is_dynamic() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_DYNAMIC);
        assert!(is_dynamic(&f));
    }

    #[test]
    fn test_multiple_flags() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_STATIC);
        set_flag(&mut f, FLAG_KINEMATIC);
        assert!(is_static(&f));
        assert!(is_kinematic(&f));
    }

    #[test]
    fn test_to_u32() {
        let mut f = new_body_flags();
        set_flag(&mut f, FLAG_STATIC);
        assert_eq!(flags_to_u32(&f), 1);
    }

    #[test]
    fn test_default_no_flags() {
        let f = new_body_flags();
        assert!(!is_static(&f));
        assert!(!is_kinematic(&f));
        assert!(!is_dynamic(&f));
    }
}
