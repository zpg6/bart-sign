//! Which platform shows on which of the two signs, and the rotation.
//!
//! Real BART signs are per platform. Most stations have two platforms, one
//! per direction, so each sign simply shows one platform. Stations with more
//! (MacArthur has four, 12th St and 19th St three) rotate: every platform
//! goes to the top sign if its trains mostly run in the `top` direction,
//! else to the bottom one, and each sign cycles through its platforms (and
//! through pages within a busy platform) one screen at a time.

use heapless::Vec;

use crate::etd::{Departures, DestGroup, Direction};
use crate::layout::{self, page_count, PageInfo};
use crate::SignMatrix;

pub const MAX_PLATFORMS: usize = 4;
const MAX_PER_PLATFORM: usize = 6;

#[derive(Clone, Debug, Default)]
pub struct Platform {
    pub number: u8,
    /// Soonest first.
    pub groups: Vec<DestGroup, MAX_PER_PLATFORM>,
}

/// Platforms for the top (index 0) and bottom (index 1) sign.
#[derive(Clone, Debug, Default)]
pub struct Board {
    pub slots: [Vec<Platform, MAX_PLATFORMS>; 2],
    /// Late evening by BART's clock: an empty sign says service is over.
    pub late: bool,
}

impl Board {
    /// Split `d` into the two signs, with `top`-bound platforms on top.
    pub fn new(d: &Departures, top: Direction) -> Board {
        let mut platforms: Vec<Platform, MAX_PLATFORMS> = Vec::new();
        // Keep the soonest-first order within each platform.
        let mut all: Vec<&DestGroup, 16> = d.south.iter().chain(d.north.iter()).collect();
        all.sort_unstable_by_key(|g| g.trains.first().map_or(u16::MAX, |t| match t.minutes {
            crate::etd::Minutes::Leaving => 0,
            crate::etd::Minutes::In(m) => m,
        }));
        for g in all {
            let i = match platforms.iter().position(|p| p.number == g.platform) {
                Some(i) => i,
                None => {
                    if platforms.push(Platform { number: g.platform, groups: Vec::new() }).is_err() {
                        continue;
                    }
                    platforms.len() - 1
                }
            };
            let _ = platforms[i].groups.push(g.clone());
        }
        platforms.sort_unstable_by_key(|p| p.number);

        let mut board = Board { late: d.is_late(), ..Board::default() };
        for p in platforms {
            let lean: i32 = p.groups.iter().map(|g| if g.direction == top { 1 } else { -1 }).sum();
            let slot = match lean {
                1.. => 0,
                ..=-1 => 1,
                // Both directions (Lake Merritt): fill the emptier sign.
                0 => usize::from(board.slots[0].len() > board.slots[1].len()),
            };
            let _ = board.slots[slot].push(p);
        }
        board
    }

    /// Screens sign `slot` cycles through.
    pub fn screens(&self, slot: usize) -> usize {
        self.slots[slot].iter().map(|p| page_count(&p.groups)).sum::<usize>().max(1)
    }

    /// Draw screen `tick` (taken modulo [`Board::screens`]) of sign `slot`.
    /// The returned platforms are what its bezel label should say; empty
    /// when the sign has no trains.
    pub fn draw(&self, m: &mut SignMatrix, slot: usize, tick: usize) -> PageInfo {
        let mut n = tick % self.screens(slot);
        for p in &self.slots[slot] {
            let pages = page_count(&p.groups);
            if n < pages {
                let mut info = layout::draw_direction_page(m, &p.groups, n);
                if info.platforms.is_empty() && p.number != 0 {
                    let _ = info.platforms.push(p.number);
                }
                return info;
            }
            n -= pages;
        }
        if self.late {
            layout::draw_message(m, "NO MORE TRAINS", "TONIGHT");
            PageInfo::default()
        } else {
            layout::draw_direction_page(m, &[], 0)
        }
    }
}
