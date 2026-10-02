struct Book {
    title: String,
    author: String,
    year: u16,
}

fn citation(book: &Book) -> String {
    let author = book.author.to_owned();
    let initial = author.chars().next().unwrap_or('?');
    format!("{} ({initial}., {})", book.title, book.year)
}

fn main() {
    let shelf = [
        Book { title: String::from("Dune"), author: String::from("Herbert"), year: 1965 },
        Book { title: String::from("Solaris"), author: String::from("Lem"), year: 1961 },
        Book { title: String::from("Kindred"), author: String::from("Butler"), year: 1979 },
    ];
    for book in &shelf {
        println!("{}", citation(book));
    }
}
