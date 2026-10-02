struct Thermostat {
    target_tenths: i32,
    heating: bool,
    switches: u32,
}

impl Thermostat {
    fn set_heating(&mut self, on: bool) {
        if self.heating != on {
            self.switches += 1;
        }
        self.heating = on;
    }

    fn observe(&mut self, tenths: i32) {
        self.set_heating(tenths < self.target_tenths);
    }
}

fn main() {
    let mut t = Thermostat { target_tenths: 205, heating: false, switches: 0 };
    for reading in [190, 198, 207, 211, 203, 199, 210] {
        t.observe(reading);
        println!("{:>5.1} C heating={}", f64::from(reading) / 10.0, t.heating);
    }
    t.set_heating(false);
    println!("switched {} times, now heating={}", t.switches, t.heating);
}
