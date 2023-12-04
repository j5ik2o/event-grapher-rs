use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub enum NameType {
  Title,
  User,
  Command,
  Event,
  Aggregate,
  Policy,
  ReadModel,
  HotSpot,
}

impl fmt::Display for NameType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      NameType::Title => write!(f, "Title"),
      NameType::User => write!(f, "User"),
      NameType::Command => write!(f, "Command"),
      NameType::Event => write!(f, "Event"),
      NameType::Aggregate => write!(f, "Aggregate"),
      NameType::Policy => write!(f, "Policy"),
      NameType::ReadModel => write!(f, "ReadModel"),
      NameType::HotSpot => write!(f, "HotSpot"),
    }
  }
}
