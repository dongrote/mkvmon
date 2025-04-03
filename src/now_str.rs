use chrono;

pub fn now_string() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M").to_string()
}
