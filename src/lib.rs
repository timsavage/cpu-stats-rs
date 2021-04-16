///
/// Read CPU statistics from proc file system
///
use std::time::Instant;
use std::io;
use std::io::BufRead;
use std::fs::File;
use std::fmt::{Display, Formatter};

const STATS_FILE: &str = "/proc/stat";

///
/// Statistics for a single CPU core, all counts are aggregates since system boot.
///
/// Times are measured in USER_HZ or Jiffies (typically hundredths of a second).
///
/// For more information see: https://www.kernel.org/doc/html/latest/filesystems/proc.html#miscellaneous-kernel-statistics-in-proc-stat
///
pub struct CoreStats {
    ///
    /// Name of core
    ///
    pub name: String,
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

macro_rules! next_value {
    ($iter:expr, $type:ty) => {
        $iter.next().and_then(|word| word.parse::<$type>().ok())
    }
}


impl CoreStats {
    fn from_str(line: &str) -> Option<Self> {
        let mut atoms = line.split(" ");

        let name = atoms.next()?;
        // If the first item is just 'cpu' it's the aggregate of all cores
        // and the next item will be a blank entry
        if name == "cpu" {
            atoms.next()?;
        }

        Some(Self {
            name: String::from(name),
            user_processes: next_value!(atoms, u64)?,
            nice_processes: next_value!(atoms, u64)?,
            system_processes: next_value!(atoms, u64)?,
            idle_time: next_value!(atoms, u64)?,
            io_wait: next_value!(atoms, u64)?,
            irq: next_value!(atoms, u64)?,
            soft_irq: next_value!(atoms, u64)?,
            steal_time: next_value!(atoms, u64)?,
            guest: next_value!(atoms, u64)?,
            guest_nice: next_value!(atoms, u64)?,
        })
    }

    ///
    /// Is the aggregate of all cores
    ///
    pub fn is_aggregate(&self) -> bool {
        self.name == "cpu"
    }

    fn diff(&self, other: &Self) -> Self {
        Self {
            name: self.name.clone(),
            user_processes: other.user_processes - self.user_processes,
            nice_processes: other.nice_processes - self.nice_processes,
            system_processes: other.system_processes - self.system_processes,
            idle_time: other.idle_time - self.idle_time,
            io_wait: other.io_wait,
            irq: other.irq - self.irq,
            soft_irq: other.soft_irq - self.soft_irq,
            steal_time: other.steal_time - self.steal_time,
            guest: other.guest - self.guest,
            guest_nice: other.guest_nice - self.guest_nice,
        }
    }
}

pub struct CoreSnapshot {
    pub stats: CoreStats,
    pub period_ms: u64,
}

impl CoreSnapshot {
    ///
    /// Percentage of last time period spent idle.
    ///
    /// Note for the aggregate this value will be greater than 100.
    ///
    pub fn idle_percent(&self) -> u64 {
        (self.stats.idle_time * 1000) / self.period_ms
    }
}

impl Display for CoreSnapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:3}%", self.stats.name, self.idle_percent())
    }
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
    pub last_stats: Vec<CoreStats>,
    ///
    /// Instant when the stats where last read
    ///
    last_instant: Instant,
}

impl CPUStatsContext {
    pub fn new() -> io::Result<Self> {
        let now = Instant::now();
        Ok(Self {
            last_stats: CPUStatsContext::raw_read()?,
            last_instant: now,
        })
    }

    ///
    /// Read stats and generate performance snapshot.
    ///
    pub fn read(&mut self) -> io::Result<Vec<CoreSnapshot>> {
        let now = Instant::now();
        let period_ms = self.last_instant.elapsed().as_millis() as u64;
        let now_stats = CPUStatsContext::raw_read()?;

        let snapshots = self.last_stats.iter().zip(&now_stats).map(|(l, n)| {
            CoreSnapshot {
                stats: l.diff(&n),
                period_ms
            }
        }).collect();

        self.last_instant = now;
        self.last_stats = now_stats;

        Ok(snapshots)
    }

    ///
    /// Read raw core stats
    ///
    fn raw_read() -> io::Result<Vec<CoreStats>> {
        let file = File::open(STATS_FILE)?;

        let mut cores: Vec<CoreStats> = Vec::new();
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            if !line.starts_with("cpu") { continue }
            if let Some(core) = CoreStats::from_str(line.as_str()) {
                cores.push(core);
            }
        }
        Ok(cores)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn refresh_stats() {
        let mut stats_context = super::CPUStatsContext::new().unwrap();

        assert!(stats_context.read().is_ok())
    }
}
