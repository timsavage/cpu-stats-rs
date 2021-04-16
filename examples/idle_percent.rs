extern crate cpu_stats;

use std::io;
use std::time::Duration;
use std::thread::sleep;
use cpu_stats::CPUStatsContext;


fn main() -> io::Result<()> {
    let mut stats = CPUStatsContext::new()?;
    let delay = Duration::from_secs(1);

    loop {
        sleep(delay);

        for core in stats.read()?.iter() {
            if !core.stats.is_aggregate() {
                print!("{:3}% ", 100 - core.idle_percent())
            }
        }
        println!()
    }
}