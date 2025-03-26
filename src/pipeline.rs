use crate::expander::Expandable;
use crate::{
    keywords,
    lexer::{Lexer, Token},
};

pub struct Pipeline {
    lang: String,
}

impl Pipeline {
    pub fn new(lang: &str) -> Pipeline {
        Pipeline {
            lang: lang.to_string(),
        }
    }

    pub fn run(&self, lexer: Lexer) -> Vec<Token> {
        lexer
            .into_iter()
            // We ignore tokens with a lexeme of length 1 since it's not
            // relevant for spelling mistakes
            .filter(|v| v.lexeme.len() > 1)
            .filter(|v| !keywords::from_lang(&self.lang).contains(v.lexeme.as_str()))
            .flat_map(|v| {
                if let Some(t) = v.expand() {
                    return t;
                }
                return vec![v];
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline() {
        let str = r#"
/**
 * Welcome to Neon!
 *
 * Neon brings simplicity, speed, and efficiency to programming. This quick introduction 
 * will guide you through some of Neon's core features and functionality.
 *
 */


// Let's declare a function

fn ignite(): string {
  "⚡ Neon is alive"  // Function returns a string automatically, no explicit return required.
}

/**
 * The function `ignite` returns a string value. 
 * Next, we'll assign that value to a variable.
 */

let signal = ignite()

// Let's demonstrate control flow with an `if` expression:

let response = if signal == "⚡ Neon is alive" {
  "💬 Status: ONLINE"  // Positive case: Status set to ONLINE.
} else {
  "Status: OFFLINE 🚨"  // Negative case: Status set to OFFLINE.
}

/**
 * In Neon, `if` statements return a value, making them usable 
 * within expressions, loops, or assignments.
 */

let final_signal = ignite() + " | " + response  // Concatenate multiple string values.

/**
 * Next, we'll use a `for` loop to iterate over an array of values.
 */

for pulse in [1, 2, 3] {

}

/**
 * Neon also supports structs. Below, we define a struct with properties:
 */

struct CyberEntity {
  name: string,
  age: int,
  location: string,
  upgrades: string[],
  status: () -> string
}

/**
 * Instantiating the struct like so:
 */

let cyber_entity = CyberEntity {
  name: "Cybernaut X",
  age: 2049,  // Define attributes of the object.
  location: "Neo Tokyo, Sector 7",
  upgrades: ["🦾 Bionic Arm", "🧠 Neural Link"],
  status: ignite
}


/**
 * Finally, we will return the avatar's status as the script's final output.
 */

cyber_entity.status()  // Evaluates to a string indicating the system's status.

/**
 * Expected output:
 * "💡 Cybernaut X is fully operational."
 *
 * Neon is built with Rust at its foundation, seamlessly integrated with WebAssembly 
 * for efficient execution in the browser. The entire system—from the lexer, parser, 
 * to the interpreter—is custom-built for a smooth and streamlined developer experience.
 *
 * Explore Neon's features further by experimenting with more examples and code variations. 
 * Thank you for exploring Neon!
 */
"#;

        let str = include_str!("../Cargo.lock");
        let lexer = Lexer::new(str);
        let tokens = Pipeline::new("rust").run(lexer);
        println!(
            "{:?}",
            tokens
                .iter()
                .map(|v| v.lexeme.clone())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
