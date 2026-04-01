use minijinja::{Environment, Value};

fn main() {
    let mut env = Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    
    let template = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\n{{ sy }}\nline 9";
    
    match env.render_str(template, &[]) {
        Ok(_) => println!("No error"),
        Err(e) => println!("Error: {}", e),
    }
}
