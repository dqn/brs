// Property ID mapping utilities ported from SkinPropertyMapper.java.

use crate::property_id::*;

/// Computes the bomb timer ID for a given player (0 or 1) and key index.
/// Returns -1 if out of range.
pub fn bomb_timer_id(player: i32, key: i32) -> i32 {
    if player < 2 {
        if key < 10 {
            return TIMER_BOMB_1P_SCRATCH + key + player * 10;
        } else if key < 100 {
            return TIMER_BOMB_1P_KEY10 + key - 10 + player * 100;
        }
    }
    -1
}

/// Computes the hold timer ID for a given player and key index.
pub fn hold_timer_id(player: i32, key: i32) -> i32 {
    if player < 2 {
        if key < 10 {
            return TIMER_HOLD_1P_SCRATCH + key + player * 10;
        } else if key < 100 {
            return TIMER_HOLD_1P_KEY10 + key - 10 + player * 100;
        }
    }
    -1
}

/// Computes the HCN active timer ID for a given player and key index.
pub fn hcn_active_timer_id(player: i32, key: i32) -> i32 {
    if player < 2 {
        if key < 10 {
            return TIMER_HCN_ACTIVE_1P_SCRATCH + key + player * 10;
        } else if key < 100 {
            return TIMER_HCN_ACTIVE_1P_KEY10 + key - 10 + player * 100;
        }
    }
    -1
}

/// Computes the HCN damage timer ID for a given player and key index.
pub fn hcn_damage_timer_id(player: i32, key: i32) -> i32 {
    if player < 2 {
        if key < 10 {
            return TIMER_HCN_DAMAGE_1P_SCRATCH + key + player * 10;
        } else if key < 100 {
            return TIMER_HCN_DAMAGE_1P_KEY10 + key - 10 + player * 100;
        }
    }
    -1
}

/// Computes the key-on timer ID for a given player and key index.
pub fn key_on_timer_id(player: i32, key: i32) -> i32 {
    if player < 2 {
        if key < 10 {
            return TIMER_KEYON_1P_SCRATCH + key + player * 10;
        } else if key < 100 {
            return TIMER_KEYON_1P_KEY10 + key - 10 + player * 100;
        }
    }
    -1
}

/// Computes the key-off timer ID for a given player and key index.
pub fn key_off_timer_id(player: i32, key: i32) -> i32 {
    if player < 2 {
        if key < 10 {
            return TIMER_KEYOFF_1P_SCRATCH + key + player * 10;
        } else if key < 100 {
            return TIMER_KEYOFF_1P_KEY10 + key - 10 + player * 100;
        }
    }
    -1
}

/// Extracts the player index from a key-judge value ID.
pub fn key_judge_value_player(value_id: i32) -> i32 {
    if (VALUE_JUDGE_1P_SCRATCH..=VALUE_JUDGE_2P_KEY9).contains(&value_id) {
        (value_id - VALUE_JUDGE_1P_SCRATCH) / 10
    } else {
        (value_id - VALUE_JUDGE_1P_KEY10) / 100
    }
}

/// Extracts the key offset from a key-judge value ID.
pub fn key_judge_value_offset(value_id: i32) -> i32 {
    if (VALUE_JUDGE_1P_SCRATCH..=VALUE_JUDGE_2P_KEY9).contains(&value_id) {
        (value_id - VALUE_JUDGE_1P_SCRATCH) % 10
    } else {
        (value_id - VALUE_JUDGE_1P_KEY10) % 100 + 10
    }
}

/// Computes the key-judge value ID for a given player and key index.
pub fn key_judge_value_id(player: i32, key: i32) -> i32 {
    if player < 2 {
        if key < 10 {
            return VALUE_JUDGE_1P_SCRATCH + key + player * 10;
        } else if key < 100 {
            return VALUE_JUDGE_1P_KEY10 + key - 10 + player * 100;
        }
    }
    -1
}

/// Returns true if the given button ID is a skin-select type button.
pub fn is_skin_select_type_id(id: i32) -> bool {
    (BUTTON_SKINSELECT_7KEY..=BUTTON_SKINSELECT_COURSE_RESULT).contains(&id)
        || (BUTTON_SKINSELECT_24KEY..=BUTTON_SKINSELECT_24KEY_BATTLE).contains(&id)
}

/// Returns the skin type ID for a skin-select button. Returns None if not a skin-select button.
pub fn skin_select_type_id(id: i32) -> Option<i32> {
    if (BUTTON_SKINSELECT_7KEY..=BUTTON_SKINSELECT_COURSE_RESULT).contains(&id) {
        Some(id - BUTTON_SKINSELECT_7KEY)
    } else if (BUTTON_SKINSELECT_24KEY..=BUTTON_SKINSELECT_24KEY_BATTLE).contains(&id) {
        Some(id - BUTTON_SKINSELECT_24KEY + 16)
    } else {
        None
    }
}

/// Computes the skin-select button ID from a skin type ID.
pub fn skin_select_button_id(skin_type_id: i32) -> i32 {
    if skin_type_id <= 15 {
        BUTTON_SKINSELECT_7KEY + skin_type_id
    } else {
        BUTTON_SKINSELECT_24KEY + skin_type_id - 16
    }
}

/// Returns true if the given button ID is a skin-customize button.
pub fn is_skin_customize_button(id: i32) -> bool {
    (BUTTON_SKIN_CUSTOMIZE1..BUTTON_SKIN_CUSTOMIZE10).contains(&id)
}

/// Returns the customize index (0-based) from a skin-customize button ID.
pub fn skin_customize_index(id: i32) -> i32 {
    id - BUTTON_SKIN_CUSTOMIZE1
}

/// Returns true if the given string ID is a skin-customize category.
pub fn is_skin_customize_category(id: i32) -> bool {
    (STRING_SKIN_CUSTOMIZE_CATEGORY1..=STRING_SKIN_CUSTOMIZE_CATEGORY10).contains(&id)
}

/// Returns the category index (0-based).
pub fn skin_customize_category_index(id: i32) -> i32 {
    id - STRING_SKIN_CUSTOMIZE_CATEGORY1
}

/// Returns true if the given string ID is a skin-customize item.
pub fn is_skin_customize_item(id: i32) -> bool {
    (STRING_SKIN_CUSTOMIZE_ITEM1..=STRING_SKIN_CUSTOMIZE_ITEM10).contains(&id)
}

/// Returns the item index (0-based).
pub fn skin_customize_item_index(id: i32) -> i32 {
    id - STRING_SKIN_CUSTOMIZE_ITEM1
}

/// Returns true if the event ID is in the custom event range.
pub fn is_custom_event_id(id: i32) -> bool {
    (EVENT_CUSTOM_BEGIN..=EVENT_CUSTOM_END).contains(&id)
}

/// Returns true if the event can be triggered by a skin.
pub fn is_event_runnable_by_skin(id: i32) -> bool {
    // Custom events are always runnable; built-in events are also allowed for now.
    if is_custom_event_id(id) {
        return true;
    }
    true
}

/// Returns true if the timer ID is in the custom timer range.
pub fn is_custom_timer_id(id: i32) -> bool {
    (TIMER_CUSTOM_BEGIN..=TIMER_CUSTOM_END).contains(&id)
}

/// Returns true if the timer can be written by a skin (only custom timers).
pub fn is_timer_writable_by_skin(id: i32) -> bool {
    is_custom_timer_id(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bomb_timer_id() {
        // 1P scratch (key=0)
        assert_eq!(bomb_timer_id(0, 0), TIMER_BOMB_1P_SCRATCH); // 50
        // 1P key1 (key=1)
        assert_eq!(bomb_timer_id(0, 1), TIMER_BOMB_1P_KEY1); // 51
        // 1P key9 (key=9)
        assert_eq!(bomb_timer_id(0, 9), TIMER_BOMB_1P_KEY9); // 59
        // 2P scratch (key=0)
        assert_eq!(bomb_timer_id(1, 0), TIMER_BOMB_2P_SCRATCH); // 60
        // 2P key7 (key=7)
        assert_eq!(bomb_timer_id(1, 7), TIMER_BOMB_2P_KEY7); // 67
        // Extended: 1P key10
        assert_eq!(bomb_timer_id(0, 10), TIMER_BOMB_1P_KEY10); // 1010
        // Extended: 1P key99
        assert_eq!(bomb_timer_id(0, 99), TIMER_BOMB_1P_KEY99); // 1099
        // Extended: 2P key10
        assert_eq!(bomb_timer_id(1, 10), TIMER_BOMB_2P_KEY10); // 1110
        // Out of range
        assert_eq!(bomb_timer_id(2, 0), -1);
        assert_eq!(bomb_timer_id(0, 100), -1);
    }

    #[test]
    fn test_hold_timer_id() {
        assert_eq!(hold_timer_id(0, 0), TIMER_HOLD_1P_SCRATCH); // 70
        assert_eq!(hold_timer_id(0, 1), TIMER_HOLD_1P_KEY1); // 71
        assert_eq!(hold_timer_id(1, 0), TIMER_HOLD_2P_SCRATCH); // 80
        assert_eq!(hold_timer_id(1, 1), TIMER_HOLD_2P_KEY1); // 81
        assert_eq!(hold_timer_id(0, 10), TIMER_HOLD_1P_KEY10); // 1210
        assert_eq!(hold_timer_id(2, 0), -1);
    }

    #[test]
    fn test_key_on_timer_id() {
        assert_eq!(key_on_timer_id(0, 0), TIMER_KEYON_1P_SCRATCH); // 100
        assert_eq!(key_on_timer_id(0, 5), TIMER_KEYON_1P_KEY5); // 105
        assert_eq!(key_on_timer_id(1, 0), TIMER_KEYON_2P_SCRATCH); // 110
        assert_eq!(key_on_timer_id(0, 10), TIMER_KEYON_1P_KEY10); // 1410
    }

    #[test]
    fn test_key_off_timer_id() {
        assert_eq!(key_off_timer_id(0, 0), TIMER_KEYOFF_1P_SCRATCH); // 120
        assert_eq!(key_off_timer_id(1, 9), TIMER_KEYOFF_2P_KEY9); // 139
        assert_eq!(key_off_timer_id(0, 10), TIMER_KEYOFF_1P_KEY10); // 1610
    }

    #[test]
    fn test_hcn_active_timer_id() {
        assert_eq!(hcn_active_timer_id(0, 0), TIMER_HCN_ACTIVE_1P_SCRATCH); // 250
        assert_eq!(hcn_active_timer_id(0, 1), TIMER_HCN_ACTIVE_1P_KEY1); // 251
        assert_eq!(hcn_active_timer_id(0, 10), TIMER_HCN_ACTIVE_1P_KEY10); // 1810
    }

    #[test]
    fn test_hcn_damage_timer_id() {
        assert_eq!(hcn_damage_timer_id(0, 0), TIMER_HCN_DAMAGE_1P_SCRATCH); // 270
        assert_eq!(hcn_damage_timer_id(0, 1), TIMER_HCN_DAMAGE_1P_KEY1); // 271
        assert_eq!(hcn_damage_timer_id(0, 10), TIMER_HCN_DAMAGE_1P_KEY10); // 2010
    }

    #[test]
    fn test_key_judge_value_id() {
        assert_eq!(key_judge_value_id(0, 0), VALUE_JUDGE_1P_SCRATCH); // 500
        assert_eq!(key_judge_value_id(0, 1), VALUE_JUDGE_1P_KEY1); // 501
        assert_eq!(key_judge_value_id(1, 0), VALUE_JUDGE_2P_SCRATCH); // 510
        assert_eq!(key_judge_value_id(0, 10), VALUE_JUDGE_1P_KEY10); // 1510
        assert_eq!(key_judge_value_id(2, 0), -1);
    }

    #[test]
    fn test_key_judge_value_player_and_offset() {
        // 1P scratch -> player=0, offset=0
        assert_eq!(key_judge_value_player(500), 0);
        assert_eq!(key_judge_value_offset(500), 0);
        // 1P key1 -> player=0, offset=1
        assert_eq!(key_judge_value_player(501), 0);
        assert_eq!(key_judge_value_offset(501), 1);
        // 2P scratch -> player=1, offset=0
        assert_eq!(key_judge_value_player(510), 1);
        assert_eq!(key_judge_value_offset(510), 0);
        // Extended: 1P key10 -> player=0, offset=10
        assert_eq!(key_judge_value_player(1510), 0);
        assert_eq!(key_judge_value_offset(1510), 10);
    }

    #[test]
    fn test_skin_select_type_mapping() {
        // 7KEY -> skin type 0
        assert!(is_skin_select_type_id(BUTTON_SKINSELECT_7KEY));
        assert_eq!(skin_select_type_id(BUTTON_SKINSELECT_7KEY), Some(0));
        // Course result -> skin type 15
        assert_eq!(
            skin_select_type_id(BUTTON_SKINSELECT_COURSE_RESULT),
            Some(15)
        );
        // 24KEY -> skin type 16
        assert_eq!(skin_select_type_id(BUTTON_SKINSELECT_24KEY), Some(16));
        // 24KEY battle -> skin type 18
        assert_eq!(
            skin_select_type_id(BUTTON_SKINSELECT_24KEY_BATTLE),
            Some(18)
        );
        // Round-trip
        assert_eq!(skin_select_button_id(0), BUTTON_SKINSELECT_7KEY);
        assert_eq!(skin_select_button_id(15), BUTTON_SKINSELECT_COURSE_RESULT);
        assert_eq!(skin_select_button_id(16), BUTTON_SKINSELECT_24KEY);
        assert_eq!(skin_select_button_id(18), BUTTON_SKINSELECT_24KEY_BATTLE);
    }

    #[test]
    fn test_skin_customize() {
        assert!(is_skin_customize_button(BUTTON_SKIN_CUSTOMIZE1));
        assert!(!is_skin_customize_button(BUTTON_SKIN_CUSTOMIZE10));
        assert_eq!(skin_customize_index(BUTTON_SKIN_CUSTOMIZE1), 0);
        assert_eq!(skin_customize_index(BUTTON_SKIN_CUSTOMIZE1 + 5), 5);
    }

    #[test]
    fn test_custom_event_and_timer() {
        assert!(is_custom_event_id(EVENT_CUSTOM_BEGIN));
        assert!(is_custom_event_id(EVENT_CUSTOM_END));
        assert!(!is_custom_event_id(EVENT_CUSTOM_BEGIN - 1));
        assert!(!is_custom_event_id(EVENT_CUSTOM_END + 1));

        assert!(is_custom_timer_id(TIMER_CUSTOM_BEGIN));
        assert!(is_custom_timer_id(TIMER_CUSTOM_END));
        assert!(!is_custom_timer_id(TIMER_MAX));

        assert!(is_timer_writable_by_skin(TIMER_CUSTOM_BEGIN));
        assert!(!is_timer_writable_by_skin(TIMER_PLAY));
    }
}
