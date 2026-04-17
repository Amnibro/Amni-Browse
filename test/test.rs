fn main() { let s = "https://google.com"; let blocked = ["sc-static.net", ""].iter().any(|d| s.contains(d)); println!("{}", blocked); }
