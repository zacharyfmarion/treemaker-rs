use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermutationError {
    InvalidDigit { digit: usize, num_digits: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermutationSnapshot {
    pub changed_digit: usize,
    pub count: usize,
    pub permutation: Vec<usize>,
}

/// Oriedita `ChainPermutationGenerator`, including persistent and temporary
/// pair guides plus top/bottom face constraints.
#[derive(Debug, Clone)]
pub struct ChainPermutationGenerator {
    count: usize,
    num_digits: usize,
    digits: Vec<usize>,
    map: Vec<usize>,
    top_indices: Option<HashSet<usize>>,
    bottom_indices: Option<HashSet<usize>>,
    swap_history: Vec<i32>,
    pair_guide: PairGuide,
    init_permutation: Vec<usize>,
    save_history: Vec<Vec<i32>>,
    is_locked: Vec<bool>,
    lock_count: usize,
    lock_remain: usize,
    saved: bool,
    restored: bool,
    looped: bool,
}

impl ChainPermutationGenerator {
    pub fn new(num_digits: usize) -> Self {
        Self {
            count: 0,
            num_digits,
            digits: vec![0; num_digits + 1],
            map: vec![0; num_digits + 1],
            top_indices: None,
            bottom_indices: None,
            swap_history: vec![0; num_digits + 1],
            pair_guide: PairGuide::new(num_digits),
            init_permutation: vec![0; num_digits + 1],
            save_history: vec![vec![0; num_digits + 1]; 3],
            is_locked: vec![false; num_digits + 1],
            lock_count: 0,
            lock_remain: 0,
            saved: false,
            restored: false,
            looped: false,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn num_digits(&self) -> usize {
        self.num_digits
    }

    pub fn locate(&self, digit: usize) -> Option<usize> {
        self.map.get(digit).copied()
    }

    pub fn permutation_at(&self, digit: usize) -> Option<usize> {
        self.digits.get(digit).copied()
    }

    pub fn current_permutation(&self) -> Vec<usize> {
        if self.num_digits == 0 {
            return Vec::new();
        }
        self.digits[1..=self.num_digits].to_vec()
    }

    pub fn snapshot(&self, changed_digit: usize) -> PermutationSnapshot {
        PermutationSnapshot {
            changed_digit,
            count: self.count,
            permutation: self.current_permutation(),
        }
    }

    pub fn add_guide(
        &mut self,
        upper_face_index: usize,
        face_index: usize,
    ) -> Result<(), PermutationError> {
        self.check_digit(upper_face_index)?;
        self.check_digit(face_index)?;
        self.pair_guide.add(upper_face_index, face_index);
        Ok(())
    }

    pub fn clear_temp_guide(&mut self) {
        self.pair_guide.clear_temp_guide(self.count != 0);
    }

    pub fn set_top_indices<I>(&mut self, top_indices: I) -> Result<(), PermutationError>
    where
        I: IntoIterator<Item = usize>,
    {
        self.top_indices = Self::validated_index_set(top_indices, self.num_digits)?;
        Ok(())
    }

    pub fn set_bottom_indices<I>(&mut self, bottom_indices: I) -> Result<(), PermutationError>
    where
        I: IntoIterator<Item = usize>,
    {
        self.bottom_indices = Self::validated_index_set(bottom_indices, self.num_digits)?;
        Ok(())
    }

    /// Lock the persistent guide graph and reset to the first valid
    /// permutation. This mirrors Oriedita's `initialize()`.
    pub fn initialize(&mut self) {
        self.is_locked.fill(false);
        if let Some(lock) = self.pair_guide.lock() {
            self.lock_count = lock[0];
            for digit in lock.iter().take(self.lock_count + 1).skip(1) {
                if let Some(is_locked) = self.is_locked.get_mut(*digit) {
                    *is_locked = true;
                }
            }

            let mut j = 1usize;
            for i in 1..=self.num_digits.saturating_sub(self.lock_count) {
                while j <= self.num_digits && self.is_locked[j] {
                    j += 1;
                }
                if j <= self.num_digits {
                    self.init_permutation[i] = j;
                    j += 1;
                }
            }
            for (i, digit) in lock.iter().enumerate().take(self.lock_count + 1).skip(1) {
                self.init_permutation[i + self.num_digits - self.lock_count] = *digit;
            }

            if let Some(last_locked) = lock.get(self.lock_count)
                && let Some(is_locked) = self.is_locked.get_mut(*last_locked)
            {
                *is_locked = false;
            }
        } else {
            self.lock_count = 1;
            for i in 1..=self.num_digits {
                self.init_permutation[i] = i;
            }
        }

        self.reset();
    }

    /// Return to the first valid permutation.
    pub fn reset(&mut self) {
        self.count = 0;
        self.lock_remain = self.lock_count;
        for i in 1..=self.num_digits {
            self.digits[i] = self.init_permutation[i];
            self.map[self.digits[i]] = i;
            if self.saved {
                self.save_history[2][i] = self.save_history[1][i];
                self.swap_history[i] = self.save_history[2][i] - 1;
            } else {
                self.swap_history[i] = i as i32 - 1;
            }
        }
        if self.saved {
            self.restored = true;
        }
        self.pair_guide.reset();
        self.next_core(1);
    }

    /// Advance the generator, returning the lowest digit changed. A return
    /// value of 0 means there is no later valid permutation.
    pub fn next(&mut self, digit: usize) -> Result<usize, PermutationError> {
        self.check_digit(digit)?;
        let result = self.next_core(digit);
        if result == 0 {
            let old_count = self.count;
            self.reset();
            self.count = old_count;
            if self.restored {
                self.looped = true;
                self.saved = false;
                self.restored = false;
                return Ok(1);
            }
            return Ok(0);
        }
        if self.looped {
            let mut i = 1usize;
            while i < self.num_digits && self.swap_history[i] == self.save_history[2][i] {
                i += 1;
            }
            if self.swap_history[i] > self.save_history[2][i] {
                self.looped = false;
                return Ok(0);
            }
        } else if self.count >= 600 && self.count.is_multiple_of(200) {
            if self.count == 800 {
                self.saved = true;
            }
            for i in 1..=self.num_digits {
                if self.count >= 800 {
                    self.save_history[1][i] = self.save_history[0][i];
                }
                self.save_history[0][i] = self.swap_history[i];
            }
        }
        Ok(result)
    }

    fn next_core(&mut self, mut digit: usize) -> usize {
        let mut cur_index = 1usize;

        if self.count > 0 {
            cur_index = self.num_digits;
            self.pair_guide.retract(self.digits[cur_index]);

            loop {
                self.swap_history[cur_index] = cur_index as i32 - 1;
                if cur_index == 0 {
                    break;
                }
                cur_index -= 1;
                self.retract(cur_index);
                if cur_index <= digit {
                    break;
                }
            }
        }

        while cur_index < self.num_digits {
            let mut swap_index = self.swap_history[cur_index];
            let mut cur_digit = 0usize;
            let max_index = self.num_digits.saturating_sub(self.lock_remain) + 1;

            loop {
                swap_index += 1;
                if swap_index < 0 || swap_index as usize > max_index {
                    break;
                }
                cur_digit = self.digits[swap_index as usize];
                if !self.pair_guide.is_not_ready(cur_digit)
                    && self.fits_constraint(cur_index, cur_digit)
                {
                    break;
                }
            }

            if swap_index < 0 || swap_index as usize > max_index {
                if self.swap_history[cur_index] == cur_index as i32 - 1
                    && !self.is_constraint_dead_end(cur_index)
                {
                    return 0;
                }

                self.swap_history[cur_index] = cur_index as i32 - 1;
                if cur_index <= 1 {
                    return 0;
                }
                cur_index -= 1;
                self.retract(cur_index);
                if cur_index < digit {
                    digit = cur_index;
                }
                continue;
            }

            let swap_index = swap_index as usize;
            if swap_index != cur_index {
                self.digits[swap_index] = self.digits[cur_index];
                self.digits[cur_index] = cur_digit;
            }
            self.swap_history[cur_index] = swap_index as i32;
            self.map[cur_digit] = cur_index;
            if self.is_locked[cur_digit] {
                self.lock_remain = self.lock_remain.saturating_sub(1);
            }
            self.pair_guide.confirm(cur_digit);

            cur_index += 1;
        }

        if self.num_digits > 0 {
            self.map[self.digits[self.num_digits]] = self.num_digits;
        }
        self.count += 1;
        digit
    }

    fn retract(&mut self, index: usize) {
        let swap_index = self.swap_history[index];
        let cur_digit = self.digits[index];
        if swap_index != index as i32 && swap_index >= 0 {
            let swap_index = swap_index as usize;
            self.digits[index] = self.digits[swap_index];
            self.digits[swap_index] = cur_digit;
        }
        self.map[cur_digit] = 0;
        if self.is_locked[cur_digit] {
            self.lock_remain += 1;
        }
        self.pair_guide.retract(cur_digit);
    }

    fn is_constraint_dead_end(&self, cur_index: usize) -> bool {
        if cur_index == 1
            && self
                .top_indices
                .as_ref()
                .is_some_and(|indices| !indices.is_empty())
        {
            return true;
        }
        cur_index == self.num_digits.saturating_sub(1)
            && self
                .bottom_indices
                .as_ref()
                .is_some_and(|indices| !indices.is_empty())
    }

    fn fits_constraint(&self, cur_index: usize, cur_digit: usize) -> bool {
        if self.num_digits == 0
            || (cur_index != 1 && cur_index != self.num_digits.saturating_sub(1))
        {
            return true;
        }
        if cur_index == 1 {
            self.top_indices
                .as_ref()
                .is_none_or(|indices| indices.contains(&cur_digit))
        } else {
            let other_digit = if cur_digit == self.digits[self.num_digits] {
                self.digits[self.num_digits - 1]
            } else {
                self.digits[self.num_digits]
            };
            self.bottom_indices
                .as_ref()
                .is_none_or(|indices| indices.contains(&other_digit))
        }
    }

    fn check_digit(&self, digit: usize) -> Result<(), PermutationError> {
        if (1..=self.num_digits).contains(&digit) {
            Ok(())
        } else {
            Err(PermutationError::InvalidDigit {
                digit,
                num_digits: self.num_digits,
            })
        }
    }

    fn validated_index_set<I>(
        indices: I,
        num_digits: usize,
    ) -> Result<Option<HashSet<usize>>, PermutationError>
    where
        I: IntoIterator<Item = usize>,
    {
        let mut set = HashSet::new();
        for digit in indices {
            if !(1..=num_digits).contains(&digit) {
                return Err(PermutationError::InvalidDigit { digit, num_digits });
            }
            set.insert(digit);
        }
        Ok((!set.is_empty()).then_some(set))
    }
}

#[derive(Debug, Clone)]
struct PairGuide {
    num_digits: usize,
    entries: Vec<usize>,
    guide: Vec<usize>,
    goal: Vec<i16>,
    score: Vec<i16>,
    locked: bool,
    added: bool,
    init_goal: Vec<i16>,
    init_guide: Vec<usize>,
    init_entries: usize,
    is_source: Vec<bool>,
    path: Vec<usize>,
    visited: Vec<usize>,
}

impl PairGuide {
    const MASK: usize = (1 << 16) - 1;

    fn new(num_digits: usize) -> Self {
        Self {
            num_digits,
            entries: vec![0],
            guide: vec![0; num_digits + 1],
            goal: vec![0; num_digits + 1],
            score: vec![0; num_digits + 1],
            locked: false,
            added: false,
            init_goal: vec![0; num_digits + 1],
            init_guide: vec![0; num_digits + 1],
            init_entries: 0,
            is_source: vec![false; num_digits + 1],
            path: vec![0; num_digits + 1],
            visited: vec![0; num_digits + 1],
        }
    }

    fn reset(&mut self) {
        for i in 1..=self.num_digits {
            self.score[i] = 0;
        }
        self.clear_temp_guide(false);
    }

    fn clear_temp_guide(&mut self, match_score: bool) {
        if self.added {
            for i in 1..=self.num_digits {
                self.guide[i] = self.init_guide[i];
                self.goal[i] = self.init_goal[i];
                if match_score {
                    self.score[i] = self.init_goal[i];
                }
            }
            self.entries.truncate(self.init_entries);
            self.added = false;
        }
    }

    fn confirm(&mut self, cur_digit: usize) {
        let mut pos = self.guide[cur_digit];
        while pos != 0 {
            let entry = self.entries[pos];
            self.score[entry & Self::MASK] += 1;
            pos = entry >> 16;
        }
    }

    fn retract(&mut self, cur_digit: usize) {
        let mut pos = self.guide[cur_digit];
        while pos != 0 {
            let entry = self.entries[pos];
            self.score[entry & Self::MASK] -= 1;
            pos = entry >> 16;
        }
    }

    fn lock(&mut self) -> Option<Vec<usize>> {
        self.locked = true;
        self.init_entries = self.entries.len();
        for i in 1..=self.num_digits {
            self.init_goal[i] = self.goal[i];
            self.init_guide[i] = self.guide[i];
        }

        let mut result = None;
        let mut max = 0usize;
        for i in 1..=self.num_digits {
            if self.is_source[i] {
                self.dfs(i, 1);
                if self.path[0] > max {
                    max = self.path[0];
                    result = Some(self.path.clone());
                    self.path.fill(0);
                }
            }
        }

        result
    }

    fn dfs(&mut self, id: usize, depth: usize) -> bool {
        if self.visited[id] > depth {
            return false;
        }
        self.visited[id] = depth;

        if self.guide[id] == 0 && depth > self.path[0] {
            self.path[0] = depth;
            self.path[depth] = id;
            return true;
        }

        let mut pos = self.guide[id];
        let mut found = false;
        while pos != 0 {
            let entry = self.entries[pos];
            if self.dfs(entry & Self::MASK, depth + 1) {
                found = true;
            }
            pos = entry >> 16;
        }
        if found {
            self.path[depth] = id;
        }
        found
    }

    fn is_not_ready(&self, cur_digit: usize) -> bool {
        self.score[cur_digit] < self.goal[cur_digit]
    }

    fn add(&mut self, upper_face_index: usize, face_index: usize) {
        let next = self.guide[upper_face_index];
        self.entries.push(face_index | (next << 16));
        self.guide[upper_face_index] = self.entries.len() - 1;
        self.goal[face_index] += 1;

        if self.locked {
            self.added = true;
            self.score[face_index] += 1;
        } else {
            self.is_source[upper_face_index] = true;
            self.is_source[face_index] = false;
        }
    }
}
