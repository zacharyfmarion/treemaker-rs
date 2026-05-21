use super::{
    AdditionalEstimationError, EquivalenceCondition, EquivalenceConditionSet, FaceOrder,
    HierarchyTable, InitialHierarchy, InitialHierarchyError, SubFace, SubFaceConfiguration,
    run_additional_estimation,
};
use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubFaceSearchError {
    Permutation(PermutationError),
    CombinationGeneratorRequired { permutation_count: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubFacePriority {
    pub ordered_subface_indices: Vec<usize>,
    pub valid_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerOverlapSearch {
    pub found: bool,
    pub hierarchy: InitialHierarchy,
    pub priority: SubFacePriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerOverlapSearchError {
    SubFace(SubFaceSearchError),
    AdditionalEstimation(AdditionalEstimationError),
    InitialHierarchy(InitialHierarchyError),
    FinalAdditionalEstimationRequired {
        valid_count: usize,
        reduced_subface_count: usize,
    },
}

impl From<SubFaceSearchError> for WorkerOverlapSearchError {
    fn from(error: SubFaceSearchError) -> Self {
        Self::SubFace(error)
    }
}

impl From<PermutationError> for WorkerOverlapSearchError {
    fn from(error: PermutationError) -> Self {
        Self::SubFace(SubFaceSearchError::Permutation(error))
    }
}

impl From<AdditionalEstimationError> for WorkerOverlapSearchError {
    fn from(error: AdditionalEstimationError) -> Self {
        Self::AdditionalEstimation(error)
    }
}

impl From<InitialHierarchyError> for WorkerOverlapSearchError {
    fn from(error: InitialHierarchyError) -> Self {
        Self::InitialHierarchy(error)
    }
}

pub fn prioritize_subfaces(
    subfaces: &[SubFace],
    reduced_subface_indices: &[usize],
    hierarchy: &InitialHierarchy,
) -> SubFacePriority {
    let reduced_count = reduced_subface_indices.len();
    let mut new_info_count = vec![0usize; reduced_count];
    let mut processed = vec![false; reduced_count];
    let mut observers = HashMap::<(usize, usize), Vec<usize>>::new();
    let mut pair_states = PairStateTable::from_hierarchy(hierarchy);

    for (reduced_index, subface_index) in reduced_subface_indices.iter().enumerate() {
        let Some(subface) = subfaces.get(*subface_index) else {
            continue;
        };
        for i in 0..subface.face_ids.len().saturating_sub(1) {
            for j in (i + 1)..subface.face_ids.len() {
                let pair = pair_key(subface.face_ids[i], subface.face_ids[j]);
                if pair_states.get(pair) == PairState::Empty {
                    observers.entry(pair).or_default().push(reduced_index);
                    new_info_count[reduced_index] += 1;
                }
            }
        }
    }

    let mut ordered_subface_indices = Vec::with_capacity(reduced_count);
    let mut valid_count = 0usize;
    for _ in 0..reduced_count {
        let (selected, max_new_info) = max_priority_subface(
            subfaces,
            reduced_subface_indices,
            &new_info_count,
            &processed,
        );
        ordered_subface_indices.push(reduced_subface_indices[selected]);
        if max_new_info > 0 {
            valid_count += 1;
        }
        processed[selected] = true;

        let Some(subface) = subfaces.get(reduced_subface_indices[selected]) else {
            continue;
        };
        for i in 0..subface.face_ids.len().saturating_sub(1) {
            for j in (i + 1)..subface.face_ids.len() {
                let pair = pair_key(subface.face_ids[i], subface.face_ids[j]);
                if pair_states.get(pair) == PairState::Empty {
                    pair_states.set(pair, PairState::Unknown);
                    if let Some(observers) = observers.get(&pair) {
                        for observer in observers {
                            new_info_count[*observer] = new_info_count[*observer].saturating_sub(1);
                        }
                    }
                }
            }
        }
    }

    SubFacePriority {
        ordered_subface_indices,
        valid_count,
    }
}

pub fn possible_overlap_search_for_subfaces(
    subfaces: &[SubFace],
    reduced_subface_indices: &[usize],
    hierarchy: &InitialHierarchy,
    conditions: Option<&EquivalenceConditionSet>,
) -> Result<WorkerOverlapSearch, WorkerOverlapSearchError> {
    let priority = prioritize_subfaces(subfaces, reduced_subface_indices, hierarchy);
    let mut searches = Vec::with_capacity(priority.ordered_subface_indices.len());
    for subface_index in &priority.ordered_subface_indices {
        let Some(subface) = subfaces.get(*subface_index) else {
            continue;
        };
        let mut search = SubFacePermutationSearch::new(subface.face_ids.clone());
        search.set_guide_map(hierarchy, conditions)?;
        searches.push(search);
    }

    let mut changed_subface = 1usize;
    while changed_subface != 0 {
        match inconsistent_subface_request(&mut searches, priority.valid_count, hierarchy)? {
            WorkerSearchStep::Consistent(table) => {
                let mut table = table;
                if let Err(error) = run_final_additional_estimation(
                    &mut table,
                    subfaces,
                    reduced_subface_indices,
                    conditions,
                ) {
                    return if priority.valid_count < priority.ordered_subface_indices.len() {
                        Err(
                            WorkerOverlapSearchError::FinalAdditionalEstimationRequired {
                                valid_count: priority.valid_count,
                                reduced_subface_count: priority.ordered_subface_indices.len(),
                            },
                        )
                    } else {
                        Err(error.into())
                    };
                }
                return Ok(WorkerOverlapSearch {
                    found: true,
                    hierarchy: table.into_initial_hierarchy(hierarchy.faces_total),
                    priority,
                });
            }
            WorkerSearchStep::Inconsistent(subface_id) => {
                changed_subface = advance_subface_permutations(&mut searches, subface_id - 1)?;
            }
        }
    }

    Ok(WorkerOverlapSearch {
        found: false,
        hierarchy: hierarchy.clone(),
        priority,
    })
}

fn run_final_additional_estimation(
    table: &mut HierarchyTable,
    subfaces: &[SubFace],
    reduced_subface_indices: &[usize],
    conditions: Option<&EquivalenceConditionSet>,
) -> Result<(), AdditionalEstimationError> {
    let configuration = SubFaceConfiguration {
        subfaces: subfaces.to_vec(),
        reduced_subface_indices: reduced_subface_indices.to_vec(),
        face_id_count_max: subfaces
            .iter()
            .map(|subface| subface.face_ids.len())
            .max()
            .unwrap_or(0),
    };
    let empty_conditions = EquivalenceConditionSet {
        triple_conditions: Vec::new(),
        quadruple_conditions: Vec::new(),
    };
    let conditions = conditions.unwrap_or(&empty_conditions);
    run_additional_estimation(
        table,
        &configuration,
        &conditions.triple_conditions,
        &conditions.quadruple_conditions,
    )
}

impl From<PermutationError> for SubFaceSearchError {
    fn from(error: PermutationError) -> Self {
        Self::Permutation(error)
    }
}

#[derive(Debug, Clone)]
pub struct SubFacePermutationSearch {
    face_ids: Vec<usize>,
    face_id_map: HashMap<usize, usize>,
    generator: ChainPermutationGenerator,
    triple_conditions: HashMap<usize, Vec<EquivalenceCondition>>,
    quadruple_conditions: Vec<EquivalenceCondition>,
}

impl SubFacePermutationSearch {
    pub fn new(face_ids: Vec<usize>) -> Self {
        let face_count = face_ids.len();
        Self {
            face_ids,
            face_id_map: HashMap::new(),
            generator: ChainPermutationGenerator::new(face_count),
            triple_conditions: HashMap::new(),
            quadruple_conditions: Vec::new(),
        }
    }

    pub fn face_ids(&self) -> &[usize] {
        &self.face_ids
    }

    pub fn permutation_count(&self) -> usize {
        self.generator.count()
    }

    pub fn current_ordering(&self) -> Vec<usize> {
        (1..=self.face_ids.len())
            .filter_map(|position| {
                let local_index = self.generator.permutation_at(position)?;
                local_index
                    .checked_sub(1)
                    .and_then(|index| self.face_ids.get(index))
                    .copied()
            })
            .collect()
    }

    pub fn next(&mut self, digit: usize) -> Result<usize, PermutationError> {
        self.generator.next(digit)
    }

    pub fn reset_permutation_generator(&mut self) {
        self.generator.reset();
    }

    /// Oriedita `SubFace.possible_overlapping_search()` without the
    /// CombinationGenerator accelerator. If that accelerator would be entered,
    /// this returns a typed unsupported error instead of approximating it.
    pub fn possible_overlapping_search(
        &mut self,
        hierarchy: &InitialHierarchy,
    ) -> Result<bool, SubFaceSearchError> {
        let table = HierarchyTable::from_initial(hierarchy);
        self.possible_overlapping_search_with_table(&table)
    }

    fn possible_overlapping_search_with_table(
        &mut self,
        table: &HierarchyTable,
    ) -> Result<bool, SubFaceSearchError> {
        let mut changed = 1usize;
        while changed != 0 {
            if self.generator.count() > 2000 {
                return Err(SubFaceSearchError::CombinationGeneratorRequired {
                    permutation_count: self.generator.count(),
                });
            }

            let inconsistent_digit = self.inconsistent_digits_request(table)?;
            if inconsistent_digit == 1000 {
                return Ok(true);
            }
            changed = self.generator.next(inconsistent_digit)?;
        }
        Ok(false)
    }

    fn enter_stacking_into(
        &self,
        table: &mut HierarchyTable,
    ) -> Result<(), AdditionalEstimationError> {
        let ordering = self.current_ordering();
        for i in 0..ordering.len().saturating_sub(1) {
            for j in (i + 1)..ordering.len() {
                table.infer_above(ordering[i], ordering[j])?;
            }
        }
        Ok(())
    }

    /// Oriedita `SubFace.setGuideMap()`: derive permutation guides from the
    /// known face hierarchy, retain equivalence conditions that are local to
    /// this subface, and initialize the generator.
    pub fn set_guide_map(
        &mut self,
        hierarchy: &InitialHierarchy,
        conditions: Option<&EquivalenceConditionSet>,
    ) -> Result<(), PermutationError> {
        let face_count = self.face_ids.len();
        self.face_id_map.clear();
        for (index, face_id) in self.face_ids.iter().enumerate() {
            self.face_id_map.insert(*face_id, index + 1);
        }

        self.generator = ChainPermutationGenerator::new(face_count);
        let table = HierarchyTable::from_initial(hierarchy);
        for face_index in 1..=face_count {
            let mut upper_face_ids = Vec::new();
            let mut upper_face_enabled = Vec::new();

            for i in 1..=face_count {
                if table.get(self.face_ids[i - 1], self.face_ids[face_index - 1])
                    == Some(FaceOrder::Above)
                {
                    upper_face_ids.push(i);
                    upper_face_enabled.push(true);
                }
            }

            for i in 0..upper_face_ids.len().saturating_sub(1) {
                for j in 0..upper_face_ids.len() {
                    if table.get(
                        self.face_ids[upper_face_ids[i] - 1],
                        self.face_ids[upper_face_ids[j] - 1],
                    ) == Some(FaceOrder::Above)
                    {
                        upper_face_enabled[i] = false;
                        break;
                    }
                }
            }

            for (i, upper_face_id) in upper_face_ids.iter().enumerate() {
                if upper_face_enabled[i] {
                    self.generator.add_guide(*upper_face_id, face_index)?;
                }
            }
        }

        self.triple_conditions.clear();
        self.quadruple_conditions.clear();
        if let Some(conditions) = conditions {
            for condition in &conditions.triple_conditions {
                if self.fast_contains(*condition) {
                    self.triple_conditions
                        .entry(condition.a)
                        .or_default()
                        .push(*condition);
                }
            }
            for condition in &conditions.quadruple_conditions {
                if self.fast_contains(*condition) {
                    self.quadruple_conditions.push(*condition);
                }
            }
        }

        self.generator.initialize();
        Ok(())
    }

    fn fast_contains(&self, condition: EquivalenceCondition) -> bool {
        self.face_id_map.contains_key(&condition.a)
            && self.face_id_map.contains_key(&condition.b)
            && self.face_id_map.contains_key(&condition.c)
            && self.face_id_map.contains_key(&condition.d)
    }

    fn inconsistent_digits_request(
        &mut self,
        hierarchy: &HierarchyTable,
    ) -> Result<usize, PermutationError> {
        let min = self.overlapping_inconsistent_digits_request(hierarchy)?;
        let min = self.penetration_inconsistent_digits_request(min);
        Ok(self.u_penetration_inconsistent_digits_request(min))
    }

    fn overlapping_inconsistent_digits_request(
        &mut self,
        hierarchy: &HierarchyTable,
    ) -> Result<usize, PermutationError> {
        let face_count = self.face_ids.len();
        for i in 1..face_count {
            for j in ((i + 1)..=face_count).rev() {
                let Some(first_local) = self.generator.permutation_at(i) else {
                    continue;
                };
                let Some(second_local) = self.generator.permutation_at(j) else {
                    continue;
                };
                let Some(first_face) = first_local
                    .checked_sub(1)
                    .and_then(|index| self.face_ids.get(index))
                    .copied()
                else {
                    continue;
                };
                let Some(second_face) = second_local
                    .checked_sub(1)
                    .and_then(|index| self.face_ids.get(index))
                    .copied()
                else {
                    continue;
                };
                if hierarchy.get(first_face, second_face) == Some(FaceOrder::Below) {
                    self.generator.add_guide(second_local, first_local)?;
                    return Ok(i);
                }
            }
        }
        Ok(1000)
    }

    fn penetration_inconsistent_digits_request(&self, min: usize) -> usize {
        for i in 1..=self.face_ids.len() {
            if i >= min {
                break;
            }
            let Some(local) = self.generator.permutation_at(i) else {
                continue;
            };
            let Some(face_id) = local
                .checked_sub(1)
                .and_then(|index| self.face_ids.get(index))
            else {
                continue;
            };
            let Some(conditions) = self.triple_conditions.get(face_id) else {
                continue;
            };
            for condition in conditions {
                if self.penetration_condition_digit(*condition, i) < min {
                    return i;
                }
            }
        }
        min
    }

    fn penetration_condition_digit(&self, condition: EquivalenceCondition, digit: usize) -> usize {
        let Some(first) = self.face_id_to_permutation_digit(condition.b) else {
            return 1000;
        };
        let Some(second) = self.face_id_to_permutation_digit(condition.d) else {
            return 1000;
        };
        if first < digit && digit < second {
            digit
        } else {
            1000
        }
    }

    fn u_penetration_inconsistent_digits_request(&self, mut min: usize) -> usize {
        for condition in &self.quadruple_conditions {
            min = self.u_penetration_condition_digit(*condition, min);
        }
        min
    }

    fn u_penetration_condition_digit(&self, condition: EquivalenceCondition, min: usize) -> usize {
        let Some(a) = self.face_id_to_permutation_digit(condition.a) else {
            return min;
        };
        let Some(b) = self.face_id_to_permutation_digit(condition.b) else {
            return min;
        };
        let Some(c) = self.face_id_to_permutation_digit(condition.c) else {
            return min;
        };
        let Some(d) = self.face_id_to_permutation_digit(condition.d) else {
            return min;
        };

        if b < min && a < c && c < b && b < d {
            return b;
        }
        if d < min && c < a && a < d && d < b {
            return d;
        }
        min
    }

    fn face_id_to_permutation_digit(&self, face_id: usize) -> Option<usize> {
        let local = self.face_id_map.get(&face_id)?;
        self.generator.locate(*local)
    }
}

enum WorkerSearchStep {
    Consistent(HierarchyTable),
    Inconsistent(usize),
}

fn inconsistent_subface_request(
    searches: &mut [SubFacePermutationSearch],
    valid_count: usize,
    hierarchy: &InitialHierarchy,
) -> Result<WorkerSearchStep, WorkerOverlapSearchError> {
    let mut table = HierarchyTable::from_initial(hierarchy);
    for index in 0..valid_count {
        let Some(search) = searches.get_mut(index) else {
            continue;
        };
        if !search.possible_overlapping_search_with_table(&table)? {
            return Ok(WorkerSearchStep::Inconsistent(index + 1));
        }
        search.enter_stacking_into(&mut table)?;
    }
    Ok(WorkerSearchStep::Consistent(table))
}

fn advance_subface_permutations(
    searches: &mut [SubFacePermutationSearch],
    subface_count: usize,
) -> Result<usize, PermutationError> {
    for search in searches.iter_mut().skip(subface_count) {
        search.reset_permutation_generator();
    }

    let mut advanced = 0usize;
    let mut subface_id = subface_count;
    for index in (0..subface_count).rev() {
        let digit_count = searches[index].face_ids.len();
        advanced = searches[index].next(digit_count)?;
        subface_id = index + 1;
        if advanced != 0 {
            break;
        }
    }
    if advanced == 0 { Ok(0) } else { Ok(subface_id) }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PairState {
    Empty,
    Unknown,
    Above,
    Below,
}

struct PairStateTable {
    states: HashMap<(usize, usize), PairState>,
}

impl PairStateTable {
    fn from_hierarchy(hierarchy: &InitialHierarchy) -> Self {
        let mut table = Self {
            states: HashMap::new(),
        };
        for relation in &hierarchy.relations {
            table.set(
                pair_key(relation.upper_face, relation.lower_face),
                if relation.upper_face < relation.lower_face {
                    PairState::Above
                } else {
                    PairState::Below
                },
            );
        }
        table
    }

    fn get(&self, pair: (usize, usize)) -> PairState {
        self.states.get(&pair).copied().unwrap_or(PairState::Empty)
    }

    fn set(&mut self, pair: (usize, usize), state: PairState) {
        self.states.insert(pair, state);
    }
}

fn max_priority_subface(
    subfaces: &[SubFace],
    reduced_subface_indices: &[usize],
    new_info_count: &[usize],
    processed: &[bool],
) -> (usize, usize) {
    let mut max_new_info = 0usize;
    let mut found = 0usize;
    for index in 0..new_info_count.len() {
        if processed[index] {
            continue;
        }
        let found_face_count = reduced_subface_indices
            .get(found)
            .and_then(|subface_index| subfaces.get(*subface_index))
            .map(|subface| subface.face_ids.len())
            .unwrap_or(0);
        let face_count = reduced_subface_indices
            .get(index)
            .and_then(|subface_index| subfaces.get(*subface_index))
            .map(|subface| subface.face_ids.len())
            .unwrap_or(0);
        if new_info_count[index] > max_new_info
            || (new_info_count[index] == max_new_info && face_count > found_face_count)
        {
            max_new_info = new_info_count[index];
            found = index;
        }
    }
    (found, max_new_info)
}

fn pair_key(first: usize, second: usize) -> (usize, usize) {
    if first <= second {
        (first, second)
    } else {
        (second, first)
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
