extern crate handlebars;
extern crate serde;

use std::fs::File;
use std::io::Write;
use std::str::FromStr;

use graphviz_rust::dot_structures::Graph;
use graphviz_rust::{
  cmd::{CommandArg, Format},
  exec_dot,
  printer::{DotPrinter, PrinterContext},
};
use handlebars::{to_json, Handlebars};
use serde_derive::Serialize;
use serde_json::value::{Map, Value as Json};

use crate::ast::name_type::NameType;
use crate::ast::Ast;

#[derive(Serialize)]
pub enum NodeType {
  User,
  Command,
  Event,
  Aggregate,
  Policy,
  ReadModel,
  HotSpot,
}

impl FromStr for NodeType {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_uppercase().as_str() {
      "USER" => Ok(NodeType::User),
      "COMMAND" => Ok(NodeType::Command),
      "EVENT" => Ok(NodeType::Event),
      "AGGREGATE" => Ok(NodeType::Aggregate),
      "POLICY" => Ok(NodeType::Policy),
      "READMODEL" => Ok(NodeType::ReadModel),
      "HOTSPOT" => Ok(NodeType::HotSpot),
      _ => Err(()),
    }
  }
}

impl From<&NameType> for NodeType {
  fn from(name_type: &NameType) -> Self {
    NodeType::from_str(&name_type.to_string()).unwrap()
  }
}

#[derive(Serialize)]
pub struct Title {
  name: String,
  caption: Option<String>,
}

impl Title {
  pub fn new(name: String, caption: Option<String>) -> Self {
    Self { name, caption }
  }
}

#[derive(Serialize)]
pub struct Node {
  name: String,
  node_type: NodeType,
  shape: String,
  fill_color: String,
  label: Option<String>,
}

impl Node {
  pub fn new(name: String, node_type: NodeType, shape: String, fill_color: String, label: Option<String>) -> Self {
    Self {
      name,
      node_type,
      shape,
      fill_color,
      label,
    }
  }
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
  label: Option<String>,
}

impl Edge {
  pub fn new(edge_type: EdgeType, from: String, to: String, label: Option<String>) -> Self {
    Self {
      edge_type,
      from,
      to,
      label,
    }
  }
}

pub struct DotWriter {
  context: Map<String, Json>,
  nodes: Vec<Node>,
  edges: Vec<Edge>,
}

impl DotWriter {
  pub fn new() -> Self {
    Self {
      context: Map::new(),
      nodes: Vec::new(),
      edges: Vec::new(),
    }
  }

  fn add_node(&mut self, node: Node) {
    self.nodes.push(node);
    let value = self.context.entry("nodes".to_string()).or_insert(to_json(&self.nodes));
    *value = to_json(&self.nodes);
  }

  fn add_edge(&mut self, edge: Edge) {
    self.edges.push(edge);
    let value = self.context.entry("edges".to_string()).or_insert(to_json(&self.edges));
    *value = to_json(&self.edges);
  }

  fn get_dot_string_from_hbs(&self) -> Result<String, Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("template", "template.hbs").unwrap();
    let out = handlebars.render("template", &self.context)?;
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

  fn fill_color(&self, ast: &Ast) -> String {
    match ast {
      Ast::NameDef(name) => match name.name_type {
        NameType::User => "lightyellow".to_string(),
        NameType::Command => "lightblue".to_string(),
        NameType::Event => "lightyellow".to_string(),
        NameType::Aggregate => "palegreen".to_string(),
        NameType::Policy => "plum".to_string(),
        NameType::ReadModel => "lightgreen".to_string(),
        NameType::HotSpot => "lightpink".to_string(),
        _ => panic!("fill_color"),
      },
      _ => panic!("fill_color"),
    }
  }

  fn shape(&self, ast: &Ast) -> String {
    match ast {
      Ast::NameDef(name) => match name.name_type {
        NameType::User => "box".to_string(),
        NameType::Command => "box".to_string(),
        NameType::Event => "box".to_string(),
        NameType::Aggregate => "box".to_string(),
        NameType::Policy => "box".to_string(),
        NameType::ReadModel => "box".to_string(),
        NameType::HotSpot => "box".to_string(),
        _ => panic!("shape"),
      },
      _ => panic!("shape"),
    }
  }

  fn eval_ast(&mut self, ast: &Ast) {
    match ast {
      Ast::TitleDef(title) => {
        let title = Title::new(title.name.clone(), title.caption.clone());
        self.context.insert("title".to_string(), to_json(title));
      }
      Ast::NameDef(name) => {
        self.add_node(Node::new(
          name.name.clone(),
          NodeType::from(&name.name_type),
          self.shape(ast),
          self.fill_color(ast),
          name.caption.clone(),
        ));
      }
      Ast::Arrow(arrow) => {
        self.add_edge(Edge::new(
          EdgeType::Arrow,
          arrow.from_ref.clone(),
          arrow.to_ref.clone(),
          arrow.caption.clone(),
        ));
      }
      Ast::Line(line) => {
        self.add_edge(Edge::new(
          EdgeType::Line,
          line.from_ref.clone(),
          line.to_ref.clone(),
          line.caption.clone(),
        ));
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
    //log::debug!("dot_string = {}", dot_string);
    // let graph = graphviz_rust::parse(&dot_string)?;

    let mut dot_file = File::create(format!("{}.dot", output_file_name))?;
    // let parsed_dot_string = self.get_parsed_dot_string(&graph)?;
    dot_file.write_all(dot_string.as_bytes())?;

    self.exec_dot(dot_string, Format::Svg, format!("{}.svg", output_file_name))?;

    Ok(())
  }
}

#[cfg(test)]
pub mod tests {
  use crate::ast::Name;
  use std::env;

  use super::*;

  #[test]
  fn it_works() {
    let mut visitor = DotWriter::new();
    let ast = Ast::TitleDef(Name::of_title("Test".to_string(), None));
    visitor.eval_ast(&ast);
    assert_eq!(visitor.context.len(), 1);
  }

  #[test]
  fn it_works2() {
    let mut visitor = DotWriter::new();
    let ast = Ast::NameDef(crate::ast::Name {
      name_type: NameType::User,
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
      Ast::TitleDef(Name::of_title("Test".to_string(), None)),
      Ast::NameDef(crate::ast::Name {
        name_type: NameType::Event,
        name: "ordered".to_string(),
        caption: Some("注文された".to_string()),
      }),
      Ast::NameDef(crate::ast::Name {
        name_type: NameType::Event,
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
        t:G:"title"

        a:Order:"注文"
        a:Inventory:"在庫"
        a:Payment:"支払い"
        a:Shipping:"出荷"

        c:OrderProduct:"商品を注文する"
        e:ProductOrdered:"商品が注文された"

        p:ReceiveTentativelyOrderPolicy:"仮受注ポリシー"

        c:ReceiveTentativelyOrder:"注文を仮受する"
        e:OrderTentativelyReceived:"注文を仮受けされた"

        p:ReserveInventoryPolicy:"在庫確保ポリシー"

        c:ReserveInventory:"在庫確保する"
        e:InventoryReserved:"在庫確保された"

        p:ProcessPaymentPolicy:"支払い処理ポリシー"

        c:ProcessPayment:"支払いを処理する"
        e:PaymentProcessed:"支払いが処理された"

        p:ValidateOrderPolicy:"注文検証ポリシー"

        c:ValidateOrder:"注文を検証する"
        e:OrderValidated:"注文が検証された"

        p:ReceiveOrderPolicy:"受注ポリシー"

        c:ReceiveOrder:"注文を請ける"
        e:OrderReceived:"注文が請けた"

        p:RequestShippingPolicy:"出荷依頼ポリシー"

        c:RequestShipping:"出荷を依頼する"
        e:ShippingRequested:"出荷が依頼された"

        OrderProduct->Order:"1"
        Order->ProductOrdered:"2"

        ProductOrdered->ReceiveTentativelyOrderPolicy:"3"
        ReceiveTentativelyOrderPolicy->ReceiveTentativelyOrder:"4"

        ReceiveTentativelyOrder->Order:"5"
        Order->OrderTentativelyReceived:"6"

        OrderTentativelyReceived->ReserveInventoryPolicy:"7"
        ReserveInventoryPolicy->ReserveInventory:"8"

        ReserveInventory->Inventory:"9"
        Inventory->InventoryReserved:"10"

        InventoryReserved->ProcessPaymentPolicy:"11"
        ProcessPaymentPolicy->ProcessPayment:"12"

        ProcessPayment->Payment:"13"
        Payment->PaymentProcessed:"14"

        PaymentProcessed->ValidateOrderPolicy:"15"
        ValidateOrderPolicy->ValidateOrder:"16"

        ValidateOrder->Order:"17"
        Order->OrderValidated:"18"

        OrderValidated->ReceiveOrderPolicy:"19"

        ReceiveOrderPolicy->ReceiveOrder:"20"
        ReceiveOrder->Order:"21"
        Order->OrderReceived:"22"

        OrderReceived->RequestShippingPolicy:"23"
        RequestShippingPolicy->RequestShipping:"24"

        RequestShipping->Shipping:"25"
        Shipping->ShippingRequested:"26"

        "#;
    let ast = crate::parsers::parse(eg.as_bytes()).unwrap();
    log::debug!("{:?}", ast);
    dot_writer.render(&ast, "target/eg").unwrap();
  }
}
