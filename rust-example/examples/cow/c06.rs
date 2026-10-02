use std::{fmt::{self, Write}, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op<'a> {
    Keep(&'a str),
    Insert(&'a str),
    Delete(&'a str),
}

impl<'a> Op<'a> {
    fn text(&self) -> &'a str {
        match *self {
            Op::Keep(s) | Op::Insert(s) | Op::Delete(s) => s,
        }
    }

    fn sigil(&self) -> char {
        match self {
            Op::Keep(_) => ' ',
            Op::Insert(_) => '+',
            Op::Delete(_) => '-',
        }
    }

    fn touches_old(&self) -> bool {
        !matches!(self, Op::Insert(_))
    }

    fn touches_new(&self) -> bool {
        !matches!(self, Op::Delete(_))
    }
}

struct Hunk<'a, 'o> {
    old_start: usize,
    new_start: usize,
    ops: &'o [Op<'a>],
}

impl<'a, 'o> Hunk<'a, 'o> {
    fn old_len(&self) -> usize {
        self.ops.iter().filter(|o| o.touches_old()).count()
    }

    fn new_len(&self) -> usize {
        self.ops.iter().filter(|o| o.touches_new()).count()
    }
}

impl<'a, 'o> fmt::Display for Hunk<'a, 'o> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "@@ -{},{} +{},{} @@", self.old_start, self.old_len(), self.new_start, self.new_len())?;
        for op in self.ops {
            writeln!(f, "{}{}", op.sigil(), op.text())?;
        }
        Ok(())
    }
}

trait Lines<'a> {
    fn line_slices(self) -> Vec<&'a str>;
}

impl<'a> Lines<'a> for &'a str {
    fn line_slices(self) -> Vec<&'a str> {
        self.lines().collect()
    }
}

fn lcs_table(a: &[&str], b: &[&str]) -> Vec<Vec<u32>> {
    let mut table = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            table[i][j] = if a[i] == b[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    table
}

fn diff<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Op<'a>> {
    let table = lcs_table(a, b);
    let (mut i, mut j) = (0, 0);
    let mut ops = Vec::with_capacity(a.len() + b.len());
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            ops.push(Op::Keep(a[i]));
            i += 1;
            j += 1;
        } else if table[i + 1][j] >= table[i][j + 1] {
            ops.push(Op::Delete(a[i]));
            i += 1;
        } else {
            ops.push(Op::Insert(b[j]));
            j += 1;
        }
    }
    ops.extend(a[i..].iter().map(|s| Op::Delete(s)));
    ops.extend(b[j..].iter().map(|s| Op::Insert(s)));
    ops
}

fn group_ranges(ops: &[Op<'_>], context: usize) -> Vec<Range<usize>> {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for (pos, _) in ops.iter().enumerate().filter(|(_, o)| !matches!(o, Op::Keep(_))) {
        let start = pos.saturating_sub(context);
        let end = (pos + context + 1).min(ops.len());
        match ranges.last_mut() {
            Some(last) if start <= last.end => last.end = last.end.max(end),
            _ => ranges.push(start..end),
        }
    }
    ranges
}

fn hunks<'a, 'o>(ops: &'o [Op<'a>], context: usize) -> Vec<Hunk<'a, 'o>> {
    group_ranges(ops, context)
        .into_iter()
        .map(|r| {
            let before = &ops[..r.start];
            Hunk {
                old_start: before.iter().filter(|o| o.touches_old()).count() + 1,
                new_start: before.iter().filter(|o| o.touches_new()).count() + 1,
                ops: &ops[r],
            }
        })
        .collect()
}

#[derive(Debug, Default, PartialEq)]
struct DiffStats {
    kept: usize,
    inserted: usize,
    deleted: usize,
}

impl<'a> FromIterator<&'a Op<'a>> for DiffStats {
    fn from_iter<I: IntoIterator<Item = &'a Op<'a>>>(iter: I) -> Self {
        iter.into_iter().fold(DiffStats::default(), |mut s, op| {
            match op {
                Op::Keep(_) => s.kept += 1,
                Op::Insert(_) => s.inserted += 1,
                Op::Delete(_) => s.deleted += 1,
            }
            s
        })
    }
}

fn render_unified(old_name: &str, new_name: &str, hunks: &[Hunk<'_, '_>]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "--- {}", old_name);
    let _ = writeln!(out, "+++ {}", new_name);
    for h in hunks {
        let _ = write!(out, "{}", h);
    }
    out
}

fn main() {
    let old = "use std::io;\n\nfn main() {\n    let x = 1;\n    let y = 2;\n    println!(\"{}\", x + y);\n}\n\nfn helper() {}\nfn unused() {}\n";
    let new = "use std::io;\nuse std::fs;\n\nfn main() {\n    let x = 1;\n    let y = 3;\n    println!(\"{}\", x + y);\n}\n\nfn helper() {}\n";
    let a = old.line_slices();
    let b = new.line_slices();
    let ops = diff(&a, &b);
    let stats: DiffStats = ops.iter().collect();
    println!("{:?}", stats);
    let hs = hunks(&ops, 1);
    print!("{}", render_unified("a/main.rs", "b/main.rs", &hs));
    let longest = ops
        .iter()
        .filter(|o| !matches!(o, Op::Keep(_)))
        .map(Op::text)
        .max_by_key(|t| t.len())
        .unwrap_or("");
    println!("longest changed line: {:?}", longest);
}
