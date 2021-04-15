///
/// Read CPU statistics from proc file system
///
use std::time::{Duration, Instant};
use std::io;
use std::fs::File;

const STATS_FILE: &str = "/proc/stats";

///
/// Statistics for a single CPU core, all counts are aggregates since system boot.
///
/// Times are measured in USER_HZ or Jiffies (typically hundredths of a second).
///
/// For more information see: https://www.kernel.org/doc/html/latest/filesystems/proc.html#miscellaneous-kernel-statistics-in-proc-stat
///
pub struct CoreStats {
    ///
    /// Normal processes executing in user mode
    ///
    pub user_processes: u64,

    ///
    /// Niced processes executing in user mode
    ///
    pub nice_processes: u64,

    ///
    /// Number of system processes executing in kernel mode
    ///
    pub system_processes: u64,

    ///
    /// Idle time
    ///
    pub idle_time: u64,

    ///
    /// I/O Waiting to complete, this item can go down!
    ///
    /// Note this is not an accurate measure, see notes in Kernel documentation linked above
    ///
    pub io_wait: u64,

    ///
    /// Servicing interrupts
    ///
    pub irq: u64,

    ///
    /// Servicing soft-interrupts
    ///
    pub soft_irq: u64,

    ///
    /// Time spent servicing virtual hosts
    ///
    pub steal_time: u64,

    ///
    /// Number of guests running
    ///
    pub guest: u64,

    ///
    /// Number of niced guests running
    ///
    pub guest_nice: u64,
}

///
/// Context for reading CPU statistics
///
/// Includes methods for getting snapshots between reads
///
pub struct CPUStatsContext {
    ///
    /// Stats from the previous read
    ///
    last_stats: Vec<CoreStats>,
    ///
    /// Instant when the stats where last read
    ///
    last_instant: Instant,
}

impl CPUStatsContext {
    pub fn new() -> io::Result<Self> {
        let now = Instant::now();
        Ok(Self {
            last_stats: CPUStatsContext::read_stats()?,
            last_instant: now,
        })
    }

    fn read_stats() -> io::Result<Vec<CoreStats>> {
        let file = File::open(STATS_FILE)?;
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
