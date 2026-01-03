use actix_session::Session;
use anyhow::{Result as AnyResult, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, Salt, SaltString, rand_core::OsRng},
};

use crate::model::*;

#[macro_export]
macro_rules! map_schema_role {
    ($conn:expr, $needle:expr, $default:expr $(,)?) => {
        $default
    };

    ($conn:expr, $needle:expr, $default:expr,
     $table_dsl:path => $table_struct:ty => $rhs:expr;
     $($rest:tt)*
    ) => {
        if $table_dsl.find($needle).first::<$table_struct>($conn).is_ok() {
            $rhs
        } else {
            map_schema_role!($conn, $needle, $default, $($rest)*)
        }
    };
}

pub fn hash_password(password: &str) -> AnyResult<(Vec<u8>, Vec<u8>)> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;

    let mut salt_bytes = [0; Salt::MAX_LENGTH];
    Ok((
        hash.hash.unwrap().as_bytes().to_vec(),
        salt.decode_b64(&mut salt_bytes)?.to_owned(),
    ))
}

pub fn verify_password(password: &str, expected_hash: &[u8], salt: &[u8]) -> AnyResult<bool> {
    let salt = SaltString::encode_b64(salt)?;
    let computed_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .hash
        .ok_or(anyhow!("Failed to get password hash"))?;

    Ok(computed_hash.as_bytes() == expected_hash)
}

pub fn is_session_authed(session: &Session) -> bool {
    session.contains_key(AUTH_INFO_SESSION_KEY)
}

pub fn is_session_admin(session: &Session) -> AnyResult<bool> {
    if let Some(auth_info) = session.get::<AuthInfo>(AUTH_INFO_SESSION_KEY)? {
        Ok(matches!(auth_info, AuthInfo::SysAdmin { .. }))
    } else {
        Err(anyhow!("Not logged in"))
    }
}

pub fn get_session_user_role(session: &Session) -> AnyResult<AuthInfoUserRole> {
    if let Some(auth_info) = session.get::<AuthInfo>(AUTH_INFO_SESSION_KEY)? {
        match auth_info {
            AuthInfo::User { role, .. } => Ok(role),
            AuthInfo::SysAdmin { .. } => Err(anyhow!("SysAdmin has no user role")),
        }
    } else {
        Err(anyhow!("Not logged in"))
    }
}
