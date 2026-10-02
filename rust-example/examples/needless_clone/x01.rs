#[derive(Debug, Clone)]
struct Recipe {
    name: String,
    servings: u32,
    ingredients: Vec<(String, f64)>,
}

fn scale(recipe: &mut Recipe, factor: f64) {
    recipe.servings = (recipe.servings as f64 * factor).round() as u32;
    for (_, qty) in recipe.ingredients.iter_mut() {
        *qty *= factor;
    }
}

fn main() {
    let base = Recipe {
        name: String::from("Pancakes"),
        servings: 4,
        ingredients: vec![(String::from("flour"), 200.0), (String::from("milk"), 300.0), (String::from("egg"), 2.0)],
    };
    let mut party = base.clone();
    scale(&mut party, 2.5);
    for r in [&base, &party] {
        println!("{} for {}:", r.name, r.servings);
        for (item, qty) in &r.ingredients {
            println!("  {item}: {qty:.1}");
        }
    }
}
