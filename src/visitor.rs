extern crate handlebars;
extern crate serde;

use std::fs::File;
use std::io::Write;
use crate::ast::Ast;
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};
use handlebars::{Handlebars, to_json};
use graphviz_rust::{
    attributes::*,
    cmd::{CommandArg, Format},
    exec, exec_dot, parse,
    printer::{DotPrinter, PrinterContext},
};

#[derive(Serialize)]
pub struct Node {
    name: String,
    caption: Option<String>,
}

#[derive(Serialize)]
pub enum EdgeType {
    Arrow,
    Line,
}

#[derive(Serialize)]
pub struct Edge {
    edge_type: EdgeType,
    from: String,
    to: String,
    caption: Option<String>,
}

pub struct DotWriter {
    data: Map<String, Json>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl DotWriter {
    pub fn new() -> Self {
        Self {
            data: Map::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn render(&mut self, ast: &Ast, output_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.eval_ast(ast);

        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_file("template", "template.hbs")
            .unwrap();
        let out = handlebars.render("template", &self.data)?;

        let mut printer_context = PrinterContext::default();
        printer_context.with_semi();

        let g = parse(&out)?;

        let dot = g.print(&mut printer_context);
        let mut dot_file = File::create(format!("{}.dot", output_file_name))?;
        dot_file.write_all(dot.as_bytes())?;


        let graph_svg = exec(
            g,
            &mut printer_context,
            vec![Format::Svg.into()],
        )?;
        let mut svg_file = File::create(format!("{}.svg", output_file_name))?;
        svg_file.write_all(graph_svg.as_bytes())?;

        log::debug!("graph_svg: {}", graph_svg);
        Ok(())
    }

    fn eval_ast(&mut self, ast: &Ast) {
        match ast {
            Ast::TitleDef(title) => {
                self.data.insert("title".to_string(), to_json(title));
            }
            Ast::NameDef(name) => {
                self.nodes.push(Node {
                    name: name.name.clone(),
                    caption: name.caption.clone(),
                });
                let m = self.data.entry("nodes".to_string()).or_insert(to_json(&self.nodes));
                *m = to_json(&self.nodes);
            }
            Ast::Arrow(arrow) => {
                self.edges.push(Edge {
                    edge_type: EdgeType::Arrow,
                    from: arrow.from_ref.clone(),
                    to: arrow.to_ref.clone(),
                    caption: arrow.caption.clone(),
                });
                let m = self.data.entry("edges".to_string()).or_insert(to_json(&self.edges));
                *m = to_json(&self.edges);
            }
            Ast::Line(line) => {
                self.edges.push(Edge {
                    edge_type: EdgeType::Line,
                    from: line.from_ref.clone(),
                    to: line.to_ref.clone(),
                    caption: line.caption.clone(),
                });
                let m = self.data.entry("edges".to_string()).or_insert(to_json(&self.edges));
                *m = to_json(&self.edges);
            }
            Ast::Comment(comment) => {}
            Ast::Empty => {}
            Ast::Documents(documents) => {
                for document in documents {
                    self.eval_ast(document);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::env;
    use super::*;

    #[test]
    fn it_works() {
        let mut visitor = DotWriter::new();
        let ast = Ast::TitleDef("Test".to_string());
        visitor.eval_ast(&ast);
        assert_eq!(visitor.data.len(), 1);
    }

    #[test]
    fn it_works2() {
        let mut visitor = DotWriter::new();
        let ast = Ast::NameDef(crate::ast::Name {
            name_type: crate::ast::NameType::User,
            name: "Test".to_string(),
            caption: None,
        });
        visitor.eval_ast(&ast);
        assert_eq!(visitor.nodes.len(), 1);
    }

    #[test]
    fn it_works3() {
        let mut visitor = DotWriter::new();
        let ast = Ast::Arrow(crate::ast::Arrow {
            from_ref: "Test".to_string(),
            to_ref: "Test".to_string(),
            caption: None,
        });
        visitor.eval_ast(&ast);
        assert_eq!(visitor.edges.len(), 1);
    }

    #[test]
    fn it_works4() {
        env::set_var("RUST_LOG", "debug");
        env_logger::init();
        let mut visitor = DotWriter::new();
        let ast = Ast::Documents(vec![
            Ast::TitleDef("Test".to_string()),
            Ast::NameDef(crate::ast::Name {
                name_type: crate::ast::NameType::Event,
                name: "ordered".to_string(),
                caption: Some("注文された".to_string()),
            }),
            Ast::NameDef(crate::ast::Name {
                name_type: crate::ast::NameType::Event,
                name: "shipping".to_string(),
                caption: Some("出荷された".to_string()),
            }),
            Ast::Arrow(crate::ast::Arrow {
                from_ref: "ordered".to_string(),
                to_ref: "shipping".to_string(),
                caption: None, //Some("XYZ".to_string()),
            })]);
        visitor.render(&ast, "target/test").unwrap();
    }
}