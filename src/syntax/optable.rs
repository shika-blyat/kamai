use std::collections::HashMap;

use super::parser::{Assoc, Fixity, Operator};

lazy_static! {
    pub static ref OPTABLE: HashMap<String, Operator<'static>> = {
        let mut map = HashMap::new();
        map.insert(
            "+".to_string(),
            Operator {
                prec: 10,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "+",
            },
        );
        map.insert(
            "-".to_string(),
            Operator {
                prec: 10,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "-",
            },
        );
        map.insert(
            "*".to_string(),
            Operator {
                prec: 20,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "*",
            },
        );
        map.insert(
            "/".to_string(),
            Operator {
                prec: 20,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "/",
            },
        );
        map.insert(
            "&&".to_string(),
            Operator {
                prec: 5,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "&&",
            },
        );
        map.insert(
            "||".to_string(),
            Operator {
                prec: 3,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "||",
            },
        );
        map.insert(
            "<".to_string(),
            Operator {
                prec: 8,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "<",
            },
        );
        map.insert(
            "<=".to_string(),
            Operator {
                prec: 8,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "<=",
            },
        );
        map.insert(
            ">".to_string(),
            Operator {
                prec: 8,
                fixity: Fixity::Infix(Assoc::Left),
                sym: ">",
            },
        );
        map.insert(
            ">=".to_string(),
            Operator {
                prec: 8,
                fixity: Fixity::Infix(Assoc::Left),
                sym: ">=",
            },
        );
        map.insert(
            "==".to_string(),
            Operator {
                prec: 8,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "==",
            },
        );
        map.insert(
            "!=".to_string(),
            Operator {
                prec: 8,
                fixity: Fixity::Infix(Assoc::Left),
                sym: "!=",
            },
        );
        map.insert(
            "!".to_string(),
            Operator {
                prec: 25,
                fixity: Fixity::Prefix,
                sym: "!",
            },
        );
        map
    };
}
