struct Signal {
    light: String,
    ticks: u32,
}

impl Signal {
    fn advance(&mut self) {
        let next = match self.light.as_str() {
            "red" => "green",
            "green" => "yellow",
            "yellow" => "red",
            _ => "red",
        };
        self.light = next.to_string();
        self.ticks += 1;
    }

    fn may_cross(&self) -> bool {
        self.light == "green"
    }
}

fn main() {
    let mut signal = Signal { light: "red".to_string(), ticks: 0 };
    for _ in 0..5 {
        signal.advance();
        println!("tick {}: {} (cross: {})", signal.ticks, signal.light, signal.may_cross());
    }
}
