struct Member {
    name: &'static str,
    role: u8,
}

fn permissions(member: &Member) -> &'static str {
    match member.role {
        0 => "read",
        1 => "read, comment",
        2 => "read, comment, edit",
        3 => "everything",
        _ => "none",
    }
}

fn can_publish(member: &Member) -> bool {
    member.role == 2 || member.role == 3
}

fn main() {
    let team = [
        Member { name: "ada", role: 3 },
        Member { name: "bo", role: 1 },
        Member { name: "cy", role: 0 },
        Member { name: "di", role: 2 },
    ];
    for m in &team {
        println!("{:<4} {:<22} publish={}", m.name, permissions(m), can_publish(m));
    }
}
