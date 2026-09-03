pub const TAB_STRIP_H: u32 = 40;
pub const NAV_H: u32 = 44;
pub const BOOKMARKS_H: u32 = 28;
pub const TOTAL_CHROME_H: u32 = TAB_STRIP_H + NAV_H + BOOKMARKS_H;
pub const CONTENT_PUSH_H: u32 = TOTAL_CHROME_H + 4;
/// Tab strip (folder tops).
pub const SERVO_TAB_H: u32 = 40;
/// Full-width omnibox row.
pub const SERVO_OMNI_H: u32 = 40;
/// Nav / actions row under the omnibox.
pub const SERVO_NAV_H: u32 = 36;
/// Load pulse strip.
pub const SERVO_PROGRESS_H: u32 = 3;
pub const SERVO_CHROME_HEIGHT_CSS: u32 =
    SERVO_TAB_H + SERVO_OMNI_H + SERVO_NAV_H + SERVO_PROGRESS_H;
pub const TAB_PAD: &str = "6px 12px";
pub const CLOSE_HITBOX: u32 = 28;
pub const NAV_HIT: u32 = 36;
