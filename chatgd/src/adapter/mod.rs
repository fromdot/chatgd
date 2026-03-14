pub mod telegram;

#[allow(dead_code)]
pub struct PlatformMessage {
    pub chat_id: i64,
    pub user_id: u64,
    pub username: Option<String>,
    pub text: String,
}
