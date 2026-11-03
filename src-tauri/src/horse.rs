use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Horse {
    pub name: String,
    pub speed: u32,
    pub stamina: u32,
}

impl Horse {
    pub fn new(name: &str, speed: u32, stamina: u32) -> Self {
        Self {
            name: name.to_string(),
            speed,
            stamina,
        }
    }
}

pub const NAME_MIN_CHARS: usize = 2;
pub const NAME_MAX_CHARS: usize = 9;

/// Racehorse naming rule (as for JRA registration): 2-9 full-width katakana,
/// where the long-vowel mark "ー" may not come first.
pub fn is_valid_name(name: &str) -> bool {
    let len = name.chars().count();
    (NAME_MIN_CHARS..=NAME_MAX_CHARS).contains(&len)
        && !name.starts_with('ー')
        && name
            .chars()
            .all(|c| ('\u{30A1}'..='\u{30F6}').contains(&c) || c == 'ー')
}

/// Two generations of ancestry shown on the 血統 screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pedigree {
    pub sire: String,
    pub dam: String,
    pub sire_sire: String,
    pub sire_dam: String,
    pub dam_sire: String,
    pub dam_dam: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names_are_2_to_9_fullwidth_katakana() {
        assert!(is_valid_name("サクラフブキ"));
        assert!(is_valid_name("オウゴンオーシャン")); // 9 chars
        assert!(is_valid_name("アサ")); // 2 chars
    }

    #[test]
    fn invalid_names_are_rejected() {
        assert!(!is_valid_name("ア")); // too short
        assert!(!is_valid_name("オウゴンオーシャンズ")); // 10 chars
        assert!(!is_valid_name("桜吹雪")); // kanji
        assert!(!is_valid_name("さくらふぶき")); // hiragana
        assert!(!is_valid_name("Sakura")); // latin
        assert!(!is_valid_name("ｻｸﾗﾌﾌﾞｷ")); // half-width kana
        assert!(!is_valid_name("ーサクラ")); // leading long-vowel mark
        assert!(!is_valid_name("サクラ ブ")); // space
        assert!(!is_valid_name(""));
    }

    #[test]
    fn test_horse_creation() {
        let horse = Horse::new("Test Horse", 80, 85);
        assert_eq!(horse.name, "Test Horse");
        assert_eq!(horse.speed, 80);
        assert_eq!(horse.stamina, 85);
    }
}
