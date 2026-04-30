//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;
use alloc::vec::Vec;
use alloc::vec;
pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;
#[allow(dead_code)]
pub struct DeadLockDetector {
    pub available: Vec<isize>,
    pub allocation: Vec<Vec<isize>>,
    pub need: Vec<Vec<isize>>,
    pub is_mutex_locked: bool,
    pub detect_enable: bool
}
#[allow(dead_code)]
pub enum ModifyType {
    Allocate,
    Release
}
#[allow(dead_code)]
impl DeadLockDetector {
    pub fn new() -> Self {
        Self {
            available: vec![],
            allocation: vec![vec![0; 16]; 16],
            need: vec![vec![0; 16]; 16],
            is_mutex_locked: false,
            detect_enable: false
        }
    }
    pub fn detect_sem(&mut self) -> bool {
        let mut work = self.available.clone();
        let mut finish = vec![false; self.allocation.len()];
        loop {
            let mut progress = false;
            for i in 0..finish.len() {
                if !finish[i] {

                    let is_satisfied = (0..self.available.len()).all(|j| self.need[i][j] <= work[j]);

                    if is_satisfied {
                        finish[i] = true;
                        progress = true;
                        for j in 0..self.available.len() {
                            work[j] += self.allocation[i][j];
                        }
                    }
                }
            }
            if !progress{
                break;
            }
        }

        if finish.iter().all(|&x| x) {

            return true;
        }

        false
    }
    pub fn modify_sem(&mut self, modify_type: ModifyType, tid: usize, sem_id: usize) {
        match modify_type {
            ModifyType::Allocate => {
                if self.available[sem_id] == 0 {
                    self.need[tid][sem_id] += 1;
                    return;
                }
                self.available[sem_id] -= 1;
                self.allocation[tid][sem_id] += 1;
            }
            ModifyType::Release => {
                self.available[sem_id] += 1;
                self.allocation[tid][sem_id] -= 1;
                return;
                
            }
        }
    }
}
