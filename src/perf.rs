use std::time::{Duration, Instant};

use tracing::debug;

pub struct Perf {
    start: Instant,
    durations: Vec<Duration>, 
    last_report: Instant,
    description: String,
}

impl Perf {
    pub fn new(description: &str) -> Self {
        Perf {
            start: Instant::now(),
            durations: vec![],
            last_report: Instant::now(),
            description: description.to_owned(),
        }
    }
    
    pub fn start(&mut self) {
        self.start = Instant::now();
    }
    
    pub fn stop(&mut self) {
        self.durations.push(self.start.elapsed());
    }
    
    pub fn report_after(&mut self, duration: Duration) {
        if self.last_report.elapsed() < duration {
            return
        }
        
        self.last_report = Instant::now();
        
        let count = self.durations.len();
        let mean = self.durations
            .iter()
            .sum::<Duration>()
            .as_secs_f32() / (count as f32);
            
        let min = self.durations
            .iter()
            .min().unwrap_or(&Duration::ZERO)
            .as_secs_f32();

        let max = self.durations
            .iter()
            .max().unwrap_or(&Duration::ZERO)
            .as_secs_f32();        
        
        debug!("{}: runs {}, mean {:.3} ms, min {:.3} ms, max {:.3} ms",
            self.description,
            count,
            mean * 1000.,
            min * 1000.,
            max * 1000.
        );
        self.durations.clear();
    }
}