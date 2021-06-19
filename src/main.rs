extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::collections::HashMap;
use std::fs;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct SystemdParser;

#[derive(Debug, Clone)]
pub enum SystemdValue {
    List(Vec<String>),
    Str(String),
}

fn pre_process_map(map: &mut HashMap<String, HashMap<String, SystemdValue>>) {
    for (_, value) in map.into_iter() {
        for (_, v) in value.into_iter() {
            if let SystemdValue::List(vs) = v {
                if vs.len() == 0 {
                    let v_ = SystemdValue::Str(String::new());
                    *v = v_.clone();
                } else if vs.len() == 1 {
                    let v_ = SystemdValue::Str((vs[0]).clone());
                    *v = v_.clone();
                }
            }
        }
    }
}

fn main() {
    let unparsed_file = fs::read_to_string("nginx.service").expect("cannot read file");
    let file = SystemdParser::parse(Rule::file, &unparsed_file)
        .expect("failed to parse")
        .next()
        .unwrap();

    let mut properties: HashMap<String, HashMap<String, SystemdValue>> = HashMap::new();

    let mut current_section_name = String::new();
    let mut current_key_name = String::new();

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::section => {
                let mut inner_rules = line.into_inner();
                current_section_name = inner_rules.next().unwrap().as_str().to_string();
            }
            Rule::property => {
                let mut inner_rules = line.into_inner();
                let section = properties.entry(current_section_name.clone()).or_default();
                let name = inner_rules.next().unwrap().as_str().to_string();
                let value = inner_rules.next().unwrap().as_str().to_string();

                if name == current_key_name {
                    let entry = section
                        .entry(current_key_name.clone())
                        .or_insert(SystemdValue::List(vec![]));
                    if let SystemdValue::List(ent) = entry {
                        ent.push(value);
                    }
                } else {
                    let entry = section
                        .entry(name.clone())
                        .or_insert(SystemdValue::List(vec![]));
                    if let SystemdValue::List(ent) = entry {
                        ent.push(value);
                    }
                    current_key_name = name;
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    pre_process_map(&mut properties);

    println!("{:?}", properties);
}
