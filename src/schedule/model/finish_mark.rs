use crate::common::datetime_utils::now_second_u32;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct FinishMarkPlace {
    task_mark: HashMap<u64, bool>,
}

impl FinishMarkPlace {
    pub fn new() -> Self {
        Self {
            task_mark: HashMap::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.task_mark.is_empty()
    }

    pub fn mark_finish(&mut self, task_id: u64, status: bool) {
        self.task_mark.insert(task_id, status);
    }

    pub fn is_finish(&self, task_id: u64) -> bool {
        self.task_mark.contains_key(&task_id)
    }

    pub fn clear(&mut self) {
        self.task_mark.clear();
    }
}

#[derive(Clone, Debug, Default)]
pub struct FinishMarkGroup {
    place_a: FinishMarkPlace,
    place_b: FinishMarkPlace,
    use_a: bool,
    next_switch_time: u32,
}

impl FinishMarkGroup {
    pub fn new() -> Self {
        Self {
            place_a: FinishMarkPlace::new(),
            place_b: FinishMarkPlace::new(),
            use_a: true,
            next_switch_time: 0,
        }
    }

    pub fn mark_finish(&mut self, task_id: u64, status: bool) {
        if self.use_a {
            self.place_a.mark_finish(task_id, status);
        } else {
            self.place_b.mark_finish(task_id, status);
        }
    }

    pub fn is_finish(&self, task_id: u64) -> bool {
        self.place_a.is_finish(task_id) || self.place_b.is_finish(task_id)
    }

    pub fn can_switch(&self, time: u32) -> bool {
        time >= self.next_switch_time
    }

    pub fn switch(&mut self, next_time: u32) {
        self.use_a = !self.use_a;
        if self.use_a {
            self.place_a.clear();
        } else {
            self.place_b.clear();
        }
        self.next_switch_time = next_time;
    }
}
