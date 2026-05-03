pub struct ProgressTable {
    processed: u64,
    success: u64,
    failed: u64,
    print_every: u64,
}

impl ProgressTable {
    pub fn new(print_every: u64) -> ProgressTable {
        ProgressTable {
            processed: 0,
            success: 0,
            failed: 0,
            print_every,
        }
    }

    pub fn record_success(&mut self) {
        self.processed += 1;
        self.success += 1;
    }

    pub fn record_failed(&mut self) {
        self.processed += 1;
        self.failed += 1;
    }

    pub fn should_print(&self) -> bool {
        self.processed == 1 || (self.print_every > 0 && self.processed % self.print_every == 0)
    }

    pub fn print_table(&self) {
        let success_rate = if self.processed == 0 {
            0.0
        } else {
            (self.success as f64 / self.processed as f64) * 100.0
        };

        println!("+----------------+----------+");
        println!("| {:<14} | {:>8} |", "processed", self.processed);
        println!("| {:<14} | {:>8} |", "success", self.success);
        println!("| {:<14} | {:>8} |", "failed", self.failed);
        println!("| {:<14} | {:>7.2}% |", "success_rate", success_rate);
        println!("+----------------+----------+");
    }

    pub fn print_final(&self) {
        self.print_table();
    }
}
