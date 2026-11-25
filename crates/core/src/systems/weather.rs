//! Weather, temperature and the season.

use crate::content::Weather;
use crate::rng::Rng;
use crate::state::{DAY, GameState, MAX_DAY};

/// Weights for each weather by season, in `Weather::ALL` order.
fn weights(day: u32) -> [u32; 6] {
    // clear, overcast, rain, fog, frost, snow
    if day < 12 {
        [30, 30, 25, 15, 0, 0]
    } else if day < 24 {
        [22, 25, 25, 15, 13, 0]
    } else if day < 34 {
        [18, 20, 15, 12, 22, 13]
    } else {
        [15, 15, 8, 10, 27, 25]
    }
}

/// Roll the weather for `day`, with some persistence from `prev`.
pub fn roll(rng: &mut Rng, day: u32, prev: Weather) -> Weather {
    let w = weights(day);
    let prev_idx = Weather::ALL.iter().position(|x| *x == prev).unwrap_or(1);
    let keep = rng.chance(35);
    if keep && w[prev_idx] > 0 {
        return prev;
    }
    let idx = rng.weighted(&w).unwrap_or(1);
    Weather::ALL[idx]
}

/// Outside temperature in whole degrees C.
pub fn temperature(s: &GameState) -> i32 {
    temperature_at(s.day(), s.minute % DAY, s.weather, s.temp_offset)
}

pub fn temperature_at(day: u32, time: u32, weather: Weather, offset: i32) -> i32 {
    // Tenths of a degree: 14.0 falling 0.55 a day (near freezing by day 25).
    let t = 140 - (day.saturating_sub(1).min(MAX_DAY) as i32) * 55 / 10;
    let hour = time / 60;
    let tod = match hour {
        0..=5 => -45,
        6..=9 => -15,
        10..=15 => 20,
        16..=19 => 0,
        _ => -30,
    };
    let w = match weather {
        Weather::Clear => {
            if !(7..19).contains(&hour) {
                -20
            } else {
                10
            }
        }
        Weather::Overcast => 10,
        Weather::Rain => 0,
        Weather::Fog => -5,
        Weather::Frost => -45,
        Weather::Snow => -25,
    };
    let total = t + tod + w + offset * 10;
    if total >= 0 { (total + 5) / 10 } else { (total - 5) / 10 }
}

/// How the radio would describe a forecast.
pub fn forecast_line(w: Weather) -> &'static str {
    match w {
        Weather::Clear => "Tomorrow: clear, a hard night after.",
        Weather::Overcast => "Tomorrow: grey and still.",
        Weather::Rain => "Tomorrow: rain from the west.",
        Weather::Fog => "Tomorrow: fog in the valleys.",
        Weather::Frost => "Tomorrow: frost, hard frost.",
        Weather::Snow => "Tomorrow: snow.",
    }
}

pub fn dawn_line(w: Weather) -> &'static str {
    match w {
        Weather::Clear => "The sky is clear.",
        Weather::Overcast => "Grey sky, low and still.",
        Weather::Rain => "Rain on the roof.",
        Weather::Fog => "Fog on the river. You cannot see the far bank.",
        Weather::Frost => "Frost on everything, white and hard.",
        Weather::Snow => "Snow, falling and lying.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn season_cools() {
        let early = temperature_at(1, 12 * 60, Weather::Overcast, 0);
        let late = temperature_at(45, 12 * 60, Weather::Overcast, 0);
        let night = temperature_at(45, 2 * 60, Weather::Frost, 0);
        assert!(early >= 13, "{early}");
        assert!(late < 0, "{late}");
        assert!(night < late - 5, "{night}");
    }

    #[test]
    fn no_snow_early() {
        let mut r = Rng::from_seed(5);
        let mut prev = Weather::Overcast;
        for _ in 0..500 {
            prev = roll(&mut r, 3, prev);
            assert!(!matches!(prev, Weather::Snow | Weather::Frost));
        }
    }
}
