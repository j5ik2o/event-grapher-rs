use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
  TitleDef(String),
  NameDef(Name),
  Arrow(Arrow),
  Line(Line),
  Comment(String),
  Empty,
  Documents(Vec<Ast>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NameType {
  User,
  Command,
  Event,
  Aggregate,
  Policy,
  ReadModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
  pub name_type: NameType,
  pub name: String,
  pub caption: Option<String>,
}

impl Name {
  pub fn of_user(name: String, caption: Option<String>) -> Self {
    Self {
      name_type: NameType::User,
      name,
      caption,
    }
  }

  pub fn of_command(name: String, caption: Option<String>) -> Self {
    Self {
      name_type: NameType::Command,
      name,
      caption,
    }
  }

  pub fn of_event(name: String, caption: Option<String>) -> Self {
    Self {
      name_type: NameType::Event,
      name,
      caption,
    }
  }

  pub fn of_aggregate(name: String, caption: Option<String>) -> Self {
    Self {
      name_type: NameType::Aggregate,
      name,
      caption,
    }
  }

  pub fn of_policy(name: String, caption: Option<String>) -> Self {
    Self {
      name_type: NameType::Policy,
      name,
      caption,
    }
  }

  pub fn of_read_model(name: String, caption: Option<String>) -> Self {
    Self {
      name_type: NameType::ReadModel,
      name,
      caption,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arrow {
  pub from_ref: String,
  pub to_ref: String,
  pub caption: Option<String>,
}

impl Arrow {
  pub fn new(from_ref: String, to_ref: String, caption: Option<String>) -> Self {
    Self { from_ref, to_ref, caption }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
  pub from_ref: String,
  pub to_ref: String,
  pub caption: Option<String>,
}

impl Line {
  pub fn new(from_ref: String, to_ref: String, caption: Option<String>) -> Self {
    Self { from_ref, to_ref, caption }
  }
}
