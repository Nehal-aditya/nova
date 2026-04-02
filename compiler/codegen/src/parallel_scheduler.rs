//! Parallel execution scheduling for NOVA
//!
//! Handles code generation for 'parallel' missions.

use crate::ir_emitter::CodegenError;

/// Parallel execution schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelStrategy {
    /// Data parallelism across array elements
    Data,
    /// Task parallelism with independent branches
    Task,
    /// Pipeline parallelism with stages
    Pipeline,
}

impl ParallelStrategy {
    pub fn to_string(&self) -> String {
        match self {
            ParallelStrategy::Data => "data_parallel".to_string(),
            ParallelStrategy::Task => "task_parallel".to_string(),
            ParallelStrategy::Pipeline => "pipeline_parallel".to_string(),
        }
    }
}

/// Scheduling information for parallel missions.
#[derive(Debug, Clone)]
pub struct ScheduleInfo {
    pub strategy: ParallelStrategy,
    pub num_threads: usize,
    pub workload_per_thread: usize,
}

impl ScheduleInfo {
    pub fn new(strategy: ParallelStrategy, num_threads: usize) -> Self {
        ScheduleInfo {
            strategy,
            num_threads,
            workload_per_thread: 1,
        }
    }
}

/// Parallel code generator.
pub struct ParallelScheduler;

impl ParallelScheduler {
    pub fn new() -> Self {
        ParallelScheduler
    }

    /// Choose a parallelization strategy based on workload.
    pub fn choose_strategy(
        &self,
        total_work: usize,
        available_threads: usize,
    ) -> Result<ScheduleInfo, CodegenError> {
        let strategy = if total_work > 10000 {
            ParallelStrategy::Data
        } else if available_threads >= 4 {
            ParallelStrategy::Task
        } else {
            ParallelStrategy::Data
        };

        Ok(ScheduleInfo::new(strategy, available_threads))
    }

    /// Emit parallel barrier synchronization.
    pub fn emit_barrier(&self) -> String {
        "call void @llvm.barrier()".to_string()
    }
}

impl Default for ParallelScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_strategy_to_string() {
        assert_eq!(ParallelStrategy::Data.to_string(), "data_parallel");
        assert_eq!(ParallelStrategy::Task.to_string(), "task_parallel");
        assert_eq!(ParallelStrategy::Pipeline.to_string(), "pipeline_parallel");
    }

    #[test]
    fn schedule_info_creation() {
        let info = ScheduleInfo::new(ParallelStrategy::Data, 8);
        assert_eq!(info.strategy, ParallelStrategy::Data);
        assert_eq!(info.num_threads, 8);
    }

    #[test]
    fn parallel_scheduler_create() {
        let scheduler = ParallelScheduler::new();
        let _ = scheduler;
    }

    #[test]
    fn choose_strategy_large_workload() {
        let scheduler = ParallelScheduler::new();
        let info = scheduler
            .choose_strategy(20000, 8)
            .expect("should choose strategy");
        assert_eq!(info.strategy, ParallelStrategy::Data);
    }

    #[test]
    fn emit_barrier() {
        let scheduler = ParallelScheduler::new();
        let barrier = scheduler.emit_barrier();
        assert!(barrier.contains("barrier"));
    }
}
