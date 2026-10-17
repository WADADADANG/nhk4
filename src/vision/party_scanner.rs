use image::{DynamicImage, GenericImageView, Rgba};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMemberState {
    pub slot: u8,
    #[serde(rename = "hpPercent")]
    pub hp_percent: f32,
    pub is_alive: bool,
    #[serde(rename = "isLowHp")]
    pub is_low_hp: bool,
    pub name: Option<String>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartyState {
    pub members: Vec<PartyMemberState>,
    #[serde(rename = "lowestHpSlot")]
    pub lowest_hp_slot: Option<u8>,
    #[serde(rename = "lowestHpPercent")]
    pub lowest_hp_percent: f32,
}

/// Pure Rust High-Performance Party HP Bar Scanner
pub struct PartyScanner;

impl PartyScanner {
    /// Check if a pixel matches the Crimson HP bar color (Red > Green * 1.4 && Red > Blue * 1.4)
    #[inline(always)]
    pub fn is_crimson_pixel(p: Rgba<u8>) -> bool {
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;
        r > 110.0 && r > g * 1.35 && r > b * 1.35
    }

    /// Scan a horizontal strip to measure the percentage of filled HP
    pub fn calculate_hp_percentage(img: &DynamicImage, x_start: u32, y: u32, width: u32) -> f32 {
        if width == 0 || x_start + width > img.width() || y >= img.height() {
            return 0.0;
        }

        let mut crimson_count = 0u32;
        let sample_steps = width.min(100);

        for step in 0..sample_steps {
            let px = x_start + (step * width / sample_steps);
            let pixel = img.get_pixel(px, y);
            if Self::is_crimson_pixel(pixel) {
                crimson_count += 1;
            }
        }

        (crimson_count as f32 / sample_steps as f32) * 100.0
    }

    /// Analyze a full frame for party slots (up to 8 members)
    pub fn scan_party(
        img: &DynamicImage,
        base_x: u32,
        base_y: u32,
        slot_height: u32,
        bar_width: u32,
        max_slots: u8,
        low_hp_threshold: f32,
    ) -> PartyState {
        let mut members = Vec::new();
        let mut lowest_hp = 100.0f32;
        let mut lowest_slot = None;

        for slot in 1..=max_slots {
            let y_pos = base_y + ((slot - 1) as u32 * slot_height);
            if y_pos >= img.height() || base_x + bar_width >= img.width() {
                break;
            }

            let hp = Self::calculate_hp_percentage(img, base_x, y_pos, bar_width);
            let is_alive = hp > 0.5;
            let is_low_hp = is_alive && hp <= low_hp_threshold;

            if is_alive && hp < lowest_hp {
                lowest_hp = hp;
                lowest_slot = Some(slot);
            }

            members.push(PartyMemberState {
                slot,
                hp_percent: hp,
                is_alive,
                is_low_hp,
                name: None,
                x: base_x,
                y: y_pos,
                width: bar_width,
                height: slot_height,
            });
        }

        PartyState {
            members,
            lowest_hp_slot: lowest_slot,
            lowest_hp_percent: if lowest_slot.is_some() { lowest_hp } else { 100.0 },
        }
    }
}
