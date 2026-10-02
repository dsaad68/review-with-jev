use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Team {
    Kitchen,
    Floor,
    Delivery,
}

struct Shift {
    worker: &'static str,
    team: Team,
    hours: f64,
}

struct PayPolicy {
    hourly_rate: f64,
    overtime_after: f64,
    overtime_multiplier: f64,
    tax_rate: f64,
}

#[derive(Debug, Default)]
struct Payslip {
    gross: f64,
    tax: f64,
    net: f64,
}

fn payslip(hours: f64, policy: &PayPolicy) -> Payslip {
    let regular = hours.min(policy.overtime_after);
    let overtime = (hours - policy.overtime_after).max(0.0);
    let gross = regular * policy.hourly_rate
        + overtime * policy.hourly_rate * policy.overtime_multiplier;
    let tax = gross * policy.tax_rate;
    Payslip { gross, tax, net: gross - tax }
}

fn main() {
    let policy = PayPolicy { hourly_rate: 18.35, overtime_after: 40.0, overtime_multiplier: 1.5, tax_rate: 0.19 };
    let shifts = [
        Shift { worker: "Ines", team: Team::Kitchen, hours: 38.5 },
        Shift { worker: "Tomas", team: Team::Floor, hours: 44.0 },
        Shift { worker: "Ravi", team: Team::Delivery, hours: 41.25 },
        Shift { worker: "Mei", team: Team::Kitchen, hours: 40.0 },
    ];

    let mut by_team: BTreeMap<Team, f64> = BTreeMap::new();
    for s in &shifts {
        let slip = payslip(s.hours, &policy);
        println!(
            "{:<6} {:>6.2}h gross {:>8.2} tax {:>7.2} net {:>8.2}",
            s.worker, s.hours, slip.gross, slip.tax, slip.net
        );
        *by_team.entry(s.team).or_insert(0.0) += slip.gross;
    }
    for (team, cost) in &by_team {
        println!("{team:?}: {cost:.2}");
    }
}
