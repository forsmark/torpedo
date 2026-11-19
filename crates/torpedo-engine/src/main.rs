//! The IBus input method engine of torpedo.

fn banner() -> String {
    format!("torpedo-engine {}", torpedo_core::version())
}

fn main() {
    println!("{}", banner());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_names_the_engine() {
        assert!(banner().starts_with("torpedo-engine "));
    }
}
