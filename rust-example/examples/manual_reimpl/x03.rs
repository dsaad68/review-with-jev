struct Recipe {
    title: &'static str,
    ingredients: Vec<&'static str>,
}

impl Recipe {
    fn shopping_line(&self) -> String {
        let mut line = String::new();
        let mut first = true;
        for item in &self.ingredients {
            if !first {
                line.push_str(", ");
            }
            line.push_str(item);
            first = false;
        }
        line
    }
}

fn main() {
    let recipe = Recipe {
        title: "shakshuka",
        ingredients: vec!["eggs", "tomatoes", "onion", "cumin", "paprika"],
    };
    println!("{}: {}", recipe.title, recipe.shopping_line());
}
