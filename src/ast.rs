#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
  Name(Name),
  Arrow(Arrow),
  Line(Line),
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
  name_type: NameType,
  name: String,
  caption: Option<String>,
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
  from: String,
  to: String,
  caption: Option<String>,
}

impl Arrow {
  pub fn new(from: String, to: String, caption: Option<String>) -> Self {
    Self { from, to, caption }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
  from: String,
  to: String,
  caption: Option<String>,
}

impl Line {
  pub fn new(from: String, to: String, caption: Option<String>) -> Self {
    Self { from, to, caption }
  }
}
