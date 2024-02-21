use time::Duration;
use time::SteadyTime;

pub struct Metronome {
    freq: Duration,
    ticked_at: SteadyTime,
}

impl Metronome {
    
    pub fn new(hz: i64) -> Metronome {
        let freq = Metronome::hz_to_duration(hz);
        Metronome { freq, ticked_at: SteadyTime::now() }
    }

    pub fn on_tick<F>(&mut self, mut f: F) 
        where F: FnMut() {
        let now = SteadyTime::now();
        if now - self.ticked_at >= self.freq {
            self.ticked_at = now;
            f();
        } 
    }

    fn hz_to_duration(hz: i64) -> Duration {
        Duration::nanoseconds(10i64.pow(9) / hz) 
    } 
    
}
