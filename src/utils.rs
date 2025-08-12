//! Houses utility functions that are used throughout the program to certain functions
//! which allow for communication with the `TradingView` server

use rand::distr::Alphanumeric;
use rand::{rng, Rng};

/// Generates a random session ID.
///
/// The session ID is a string in the format `prefix_random_string`, where `prefix` is an optional
/// parameter and `random_string` is a 12-character alphanumeric string. If `prefix` is not
/// provided by passing `None`, the default prefix "xs" is used.
///
/// Returns a randomly generated session id
///
/// # Examples
///
/// ```
/// use trade_vision::utils::generate_session_id;
/// let session_id = generate_session_id(None);
/// assert!(session_id.starts_with("qs_"));
///
/// let session_id = generate_session_id(Some("foo"));
/// assert!(session_id.starts_with("foo_"));
/// ```
///
pub fn generate_session_id(prefix: Option<&str>) -> String {
    let mut rng = rng();
    let random_string: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    format!("{}_{}", prefix.unwrap_or("qs"), random_string)
}

#[test]
fn test_generate_session_id() {
    let session_id = generate_session_id(None);
    assert!(
        session_id.starts_with("qs_"),
        "Expected prefix 'qs_', got {session_id}"
    );
    assert_eq!(
        session_id.len(),
        15,
        "Expected length 15, got {}",
        session_id.len()
    );

    let session_id = generate_session_id(Some("foo"));
    assert!(
        session_id.starts_with("foo_"),
        "Expected prefix 'foo_', got {session_id}"
    );
    assert_eq!(
        session_id.len(),
        16,
        "Expected length 16, got {}",
        session_id.len()
    );
}
