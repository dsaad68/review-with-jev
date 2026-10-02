struct Account {
    balance_cents: u64,
    withdrawn_today_cents: u64,
    daily_limit_cents: u64,
}

impl Account {
    fn withdraw(&mut self, amount_cents: u64) -> Result<u64, &'static str> {
        if amount_cents == 0 {
            return Err("withdrawal amount must be positive");
        }
        if amount_cents > self.balance_cents {
            return Err("insufficient funds");
        }
        if self.withdrawn_today_cents + amount_cents > self.daily_limit_cents {
            return Err("daily withdrawal limit reached");
        }
        self.balance_cents -= amount_cents;
        self.withdrawn_today_cents += amount_cents;
        Ok(self.balance_cents)
    }
}

fn main() {
    let mut account = Account { balance_cents: 50_000, withdrawn_today_cents: 0, daily_limit_cents: 30_000 };
    for amount in [20_000, 0, 15_000, 60_000, 10_000] {
        match account.withdraw(amount) {
            Ok(left) => println!("withdrew {amount}, balance {left}"),
            Err(reason) => println!("declined {amount}: {reason}"),
        }
    }
}
