//! Identité : comptes, connexion, sessions, droits (F01, F02 ; DP-85). Le seul endroit où la matrice des droits est codée.

pub mod account;
pub mod admin;
pub mod audit_read;
pub mod authorize;
pub mod crypto;
pub mod error;
pub mod identifier;
pub mod members;
pub mod observe;
pub mod password;
pub mod service;
pub mod session;
pub mod settings;
pub mod support;
pub mod throttle;
pub mod totp;
