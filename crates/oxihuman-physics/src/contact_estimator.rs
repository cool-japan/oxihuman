// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Contact state estimator stub.

/// Contact state for a single foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactState {
    NoContact,
    SoftContact,
    HardContact,
}

/// Contact estimate for a foot.
#[derive(Debug, Clone)]
pub struct FootContact {
    pub foot_id: usize,
    pub state: ContactState,
    pub normal_force: f32,
    pub position: [f32; 3],
}

impl FootContact {
    pub fn no_contact(foot_id: usize) -> Self {
        Self {
            foot_id,
            state: ContactState::NoContact,
            normal_force: 0.0,
            position: [0.0; 3],
        }
    }
}

/// Contact estimator configuration.
#[derive(Debug, Clone)]
pub struct ContactEstimatorConfig {
    pub soft_contact_threshold: f32,
    pub hard_contact_threshold: f32,
    pub force_alpha: f32,
}

impl Default for ContactEstimatorConfig {
    fn default() -> Self {
        Self {
            soft_contact_threshold: 5.0,
            hard_contact_threshold: 50.0,
            force_alpha: 0.3,
        }
    }
}

/// Estimate contact state from measured normal force.
pub fn estimate_contact_state(force: f32, cfg: &ContactEstimatorConfig) -> ContactState {
    if force < cfg.soft_contact_threshold {
        ContactState::NoContact
    } else if force < cfg.hard_contact_threshold {
        ContactState::SoftContact
    } else {
        ContactState::HardContact
    }
}

/// Update a foot contact estimate with a new force measurement.
pub fn update_foot_contact(
    contact: &mut FootContact,
    raw_force: f32,
    position: [f32; 3],
    cfg: &ContactEstimatorConfig,
) {
    /* low-pass filter the force */
    contact.normal_force =
        (1.0 - cfg.force_alpha) * contact.normal_force + cfg.force_alpha * raw_force;
    contact.state = estimate_contact_state(contact.normal_force, cfg);
    contact.position = position;
}

/// Return the total ground reaction force from all contacts.
pub fn total_grf(contacts: &[FootContact]) -> f32 {
    contacts.iter().map(|c| c.normal_force).sum()
}

/// Return the number of feet currently in contact.
pub fn contact_count(contacts: &[FootContact]) -> usize {
    contacts
        .iter()
        .filter(|c| c.state != ContactState::NoContact)
        .count()
}

/// Return whether any foot is in hard contact.
pub fn any_hard_contact(contacts: &[FootContact]) -> bool {
    contacts
        .iter()
        .any(|c| c.state == ContactState::HardContact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_contact_low_force() {
        /* low force → no contact */
        let cfg = ContactEstimatorConfig::default();
        assert_eq!(estimate_contact_state(1.0, &cfg), ContactState::NoContact);
    }

    #[test]
    fn test_soft_contact() {
        /* medium force → soft contact */
        let cfg = ContactEstimatorConfig::default();
        assert_eq!(
            estimate_contact_state(20.0, &cfg),
            ContactState::SoftContact
        );
    }

    #[test]
    fn test_hard_contact() {
        /* high force → hard contact */
        let cfg = ContactEstimatorConfig::default();
        assert_eq!(
            estimate_contact_state(100.0, &cfg),
            ContactState::HardContact
        );
    }

    #[test]
    fn test_update_contact_state() {
        /* large force updates to hard contact */
        let cfg = ContactEstimatorConfig::default();
        let mut c = FootContact::no_contact(0);
        /* many updates to converge */
        for _ in 0..30 {
            update_foot_contact(&mut c, 200.0, [0.0; 3], &cfg);
        }
        assert_eq!(c.state, ContactState::HardContact);
    }

    #[test]
    fn test_total_grf_zero() {
        /* no contacts → zero GRF */
        let contacts = vec![FootContact::no_contact(0), FootContact::no_contact(1)];
        assert_eq!(total_grf(&contacts), 0.0);
    }

    #[test]
    fn test_contact_count() {
        /* count contacts correctly */
        let mut contacts = vec![FootContact::no_contact(0), FootContact::no_contact(1)];
        contacts[0].state = ContactState::HardContact;
        assert_eq!(contact_count(&contacts), 1);
    }

    #[test]
    fn test_any_hard_contact_false() {
        /* no hard contact initially */
        let contacts = vec![FootContact::no_contact(0)];
        assert!(!any_hard_contact(&contacts));
    }

    #[test]
    fn test_any_hard_contact_true() {
        /* hard contact detected */
        let mut c = FootContact::no_contact(0);
        c.state = ContactState::HardContact;
        assert!(any_hard_contact(&[c]));
    }

    #[test]
    fn test_no_contact_initial() {
        /* initial state is NoContact */
        let c = FootContact::no_contact(0);
        assert_eq!(c.state, ContactState::NoContact);
    }
}
