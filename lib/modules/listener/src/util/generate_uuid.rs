use uuid::Uuid;

pub fn generate_session_key() -> String {
    Uuid::new_v4().to_string()
}
 