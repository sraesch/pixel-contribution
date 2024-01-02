/// A progress bar for the console.
pub struct Progress {
    cur: usize,
    total: usize,
}

impl Progress {
    /// Creates a new progress bar with the given total number of steps.
    pub fn new(total: usize) -> Self {
        Self { cur: 0, total }
    }

    /// Updates the progress of the current task and prints it to the console.
    pub fn update(&mut self) {
        self.cur += 1;
        self.print();
    }

    /// Prints the progress of the current task to the console.
    pub fn print(&self) {
        let bar_length = 50;
        let progress = self.cur as f32 / self.total as f32;
        let num_bars = (progress * bar_length as f32) as usize;
        let num_spaces = bar_length - num_bars;

        print!("\r[");
        for _ in 0..num_bars {
            print!("=");
        }
        for _ in 0..num_spaces {
            print!(" ");
        }
        print!("] {:.2}%", progress * 100.0);
    }
}
