use std::fmt;

use crate::{CLIPS_PER_ROW, Clip, ClipLauncher, PATTERN_ROWS, PATTERN_STEPS};

pub const REGIONS_PER_ROW: usize = 32;
pub const ARRANGEMENT_MAX_BARS: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrangementError {
    InvalidRow,
    InvalidRegionSlot,
    InvalidClipSlot,
    InvalidRange,
    Overlap,
    MissingClip,
}

impl fmt::Display for ArrangementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRow => "arrangement row must be between zero and three",
            Self::InvalidRegionSlot => "region slot must be between zero and thirty-one",
            Self::InvalidClipSlot => "clip slot must be between zero and fifteen",
            Self::InvalidRange => "region must have positive length within 4096 bars",
            Self::Overlap => "regions in the same row must not overlap",
            Self::MissingClip => "arrangement region references an empty clip slot",
        })
    }
}

impl std::error::Error for ArrangementError {}

/// Half-open bar interval referencing a clip in the region's own row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRegion {
    clip_slot: u8,
    start_bar: u16,
    length_bars: u16,
}

impl ClipRegion {
    pub fn new(
        clip_slot: usize,
        start_bar: usize,
        length_bars: usize,
    ) -> Result<Self, ArrangementError> {
        if clip_slot >= CLIPS_PER_ROW {
            return Err(ArrangementError::InvalidClipSlot);
        }
        if length_bars == 0
            || start_bar >= ARRANGEMENT_MAX_BARS
            || length_bars > ARRANGEMENT_MAX_BARS - start_bar
        {
            return Err(ArrangementError::InvalidRange);
        }
        Ok(Self {
            clip_slot: clip_slot as u8,
            start_bar: start_bar as u16,
            length_bars: length_bars as u16,
        })
    }

    pub const fn clip_slot(&self) -> usize {
        self.clip_slot as usize
    }

    pub const fn start_bar(&self) -> usize {
        self.start_bar as usize
    }

    pub const fn length_bars(&self) -> usize {
        self.length_bars as usize
    }

    pub const fn end_bar(&self) -> usize {
        self.start_bar() + self.length_bars()
    }

    pub const fn start_step(&self) -> u64 {
        self.start_bar() as u64 * PATTERN_STEPS as u64
    }

    pub const fn end_step(&self) -> u64 {
        self.end_bar() as u64 * PATTERN_STEPS as u64
    }

    fn overlaps(&self, other: Self) -> bool {
        self.start_bar() < other.end_bar() && other.start_bar() < self.end_bar()
    }
}

/// Editable fixed-capacity timeline. Slot identity is independent of time order.
/// This standalone document is not yet part of Project or its file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arrangement {
    rows: [[Option<ClipRegion>; REGIONS_PER_ROW]; PATTERN_ROWS],
}

impl Default for Arrangement {
    fn default() -> Self {
        Self {
            rows: [[None; REGIONS_PER_ROW]; PATTERN_ROWS],
        }
    }
}

impl Arrangement {
    pub fn region(&self, row: usize, slot: usize) -> Result<Option<&ClipRegion>, ArrangementError> {
        check_cell(row, slot)?;
        Ok(self.rows[row][slot].as_ref())
    }

    /// Validates before replacement; edits to a region keep its slot identity.
    pub fn set_region(
        &mut self,
        row: usize,
        slot: usize,
        region: ClipRegion,
        launcher: &ClipLauncher,
    ) -> Result<(), ArrangementError> {
        check_cell(row, slot)?;
        referenced_clip(launcher, row, region)?;
        if self.rows[row]
            .iter()
            .enumerate()
            .any(|(index, other)| index != slot && other.is_some_and(|r| r.overlaps(region)))
        {
            return Err(ArrangementError::Overlap);
        }
        self.rows[row][slot] = Some(region);
        Ok(())
    }

    pub fn remove_region(&mut self, row: usize, slot: usize) -> Result<(), ArrangementError> {
        check_cell(row, slot)?;
        self.rows[row][slot] = None;
        Ok(())
    }

    pub fn references_clip(&self, row: usize, clip_slot: usize) -> Result<bool, ArrangementError> {
        let regions = self.rows.get(row).ok_or(ArrangementError::InvalidRow)?;
        if clip_slot >= CLIPS_PER_ROW {
            return Err(ArrangementError::InvalidClipSlot);
        }
        Ok(regions.iter().flatten().any(|r| r.clip_slot() == clip_slot))
    }

    /// Last exclusive end, including leading/intermediate gaps; empty is zero.
    pub fn duration_bars(&self) -> usize {
        self.rows
            .iter()
            .flatten()
            .flatten()
            .map(ClipRegion::end_bar)
            .max()
            .unwrap_or(0)
    }

    /// Revalidates references and owns clip values. Later source edits cannot
    /// change this snapshot. No partially resolved result escapes on failure.
    pub fn resolve(
        &self,
        launcher: &ClipLauncher,
    ) -> Result<ResolvedArrangement, ArrangementError> {
        let mut resolved = ResolvedArrangement {
            rows: [[None; REGIONS_PER_ROW]; PATTERN_ROWS],
            duration_steps: self.duration_bars() as u64 * PATTERN_STEPS as u64,
        };
        for (row, regions) in self.rows.iter().enumerate() {
            for (slot, region) in regions.iter().enumerate() {
                if let Some(region) = region {
                    resolved.rows[row][slot] = Some(ResolvedRegion {
                        region: *region,
                        clip: referenced_clip(launcher, row, *region)?,
                    });
                }
            }
        }
        Ok(resolved)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedRegion {
    region: ClipRegion,
    clip: Clip,
}

impl ResolvedRegion {
    pub const fn region(&self) -> &ClipRegion {
        &self.region
    }

    pub const fn clip(&self) -> &Clip {
        &self.clip
    }

    /// Region-local sixteenth note, or None outside the half-open interval.
    pub fn local_step_at(&self, global_step: u64) -> Option<usize> {
        if global_step < self.region.start_step() || global_step >= self.region.end_step() {
            return None;
        }
        Some(((global_step - self.region.start_step()) % PATTERN_STEPS as u64) as usize)
    }
}

/// Immutable domain snapshot, not a realtime command or a live playhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedArrangement {
    rows: [[Option<ResolvedRegion>; REGIONS_PER_ROW]; PATTERN_ROWS],
    duration_steps: u64,
}

impl ResolvedArrangement {
    pub const fn duration_steps(&self) -> u64 {
        self.duration_steps
    }

    pub fn region(
        &self,
        row: usize,
        slot: usize,
    ) -> Result<Option<&ResolvedRegion>, ArrangementError> {
        check_cell(row, slot)?;
        Ok(self.rows[row][slot].as_ref())
    }

    pub fn region_at_step(
        &self,
        row: usize,
        global_step: u64,
    ) -> Result<Option<&ResolvedRegion>, ArrangementError> {
        let regions = self.rows.get(row).ok_or(ArrangementError::InvalidRow)?;
        Ok(regions
            .iter()
            .flatten()
            .find(|region| region.local_step_at(global_step).is_some()))
    }
}

fn check_cell(row: usize, slot: usize) -> Result<(), ArrangementError> {
    if row >= PATTERN_ROWS {
        return Err(ArrangementError::InvalidRow);
    }
    if slot >= REGIONS_PER_ROW {
        return Err(ArrangementError::InvalidRegionSlot);
    }
    Ok(())
}

fn referenced_clip(
    launcher: &ClipLauncher,
    row: usize,
    region: ClipRegion,
) -> Result<Clip, ArrangementError> {
    launcher
        .clip(row, region.clip_slot())
        .expect("validated row and clip slot")
        .copied()
        .ok_or(ArrangementError::MissingClip)
}
