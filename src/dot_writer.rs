extern crate handlebars;
extern crate serde;

use std::fs::File;
use std::io::Write;

use graphviz_rust::dot_structures::Graph;
use graphviz_rust::{
  cmd::{CommandArg, Format},
  exec_dot,
  printer::{DotPrinter, PrinterContext},
};
use handlebars::{to_json, Handlebars};
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};

use crate::ast::Ast;

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

  fn add_node(&mut self, node: Node) {
    self.nodes.push(node);
    let m = self.data.entry("nodes".to_string()).or_insert(to_json(&self.nodes));
    *m = to_json(&self.nodes);
  }

  fn add_edge(&mut self, edge: Edge) {
    self.edges.push(edge);
    let m = self.data.entry("edges".to_string()).or_insert(to_json(&self.edges));
    *m = to_json(&self.edges);
  }

  fn get_dot_string_from_hbs(&self) -> Result<String, Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("template", "template.hbs").unwrap();
    let out = handlebars.render("template", &self.data)?;
    Ok(out)
  }

  fn get_parsed_dot_string(&self, g: &Graph) -> Result<String, Box<dyn std::error::Error>> {
    let mut printer_context = PrinterContext::default();
    printer_context.with_semi();
    let dot = g.print(&mut printer_context);
    Ok(dot)
  }

  fn exec_dot(&self, dot_string: String, fmt: Format, output: String) -> Result<(), Box<dyn std::error::Error>> {
    exec_dot(dot_string, vec![CommandArg::Format(fmt), CommandArg::Output(output)])?;
    Ok(())
  }

  fn eval_ast(&mut self, ast: &Ast) {
    match ast {
      Ast::TitleDef(title) => {
        self.data.insert("title".to_string(), to_json(title));
      }
      Ast::NameDef(name) => {
        self.add_node(Node {
          name: name.name.clone(),
          caption: name.caption.clone(),
        });
      }
      Ast::Arrow(arrow) => {
        self.add_edge(Edge {
          edge_type: EdgeType::Arrow,
          from: arrow.from_ref.clone(),
          to: arrow.to_ref.clone(),
          caption: arrow.caption.clone(),
        });
      }
      Ast::Line(line) => {
        self.add_edge(Edge {
          edge_type: EdgeType::Line,
          from: line.from_ref.clone(),
          to: line.to_ref.clone(),
          caption: line.caption.clone(),
        });
      }
      Ast::Documents(documents) => {
        for document in documents {
          self.eval_ast(document);
        }
      }
      Ast::Comment(comment) => {}
      Ast::Empty => {}
    }
  }

  pub fn render(&mut self, ast: &Ast, output_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    self.eval_ast(ast);

    let dot_string = self.get_dot_string_from_hbs()?;
    log::debug!("dot_string = {}", dot_string);
    let graph = graphviz_rust::parse(&dot_string)?;

    let mut dot_file = File::create(format!("{}.dot", output_file_name))?;
    let parsed_dot_string = self.get_parsed_dot_string(&graph)?;
    dot_file.write_all(parsed_dot_string.as_bytes())?;

    self.exec_dot(parsed_dot_string, Format::Svg, format!("{}.svg", output_file_name))?;

    Ok(())
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
    let mut dot_writer = DotWriter::new();
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
      }),
    ]);
    dot_writer.render(&ast, "target/test").unwrap();
  }

  #[test]
  fn test() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let mut dot_writer = DotWriter::new();
    let eg = r#"
        t:"title"
        e:InventoryReserved:"在庫確保された"
        e:PaymentProcessed:"決済処理された"
        e:ShipmentScheduled:"出荷スケジュールされた"
        InventoryReserved->PaymentProcessed
        PaymentProcessed->ShipmentScheduled
        "#;
    let ast = crate::parsers::parse(eg.as_bytes()).unwrap();
    log::debug!("{:?}", ast);
    dot_writer.render(&ast, "target/eg").unwrap();
  }
}
