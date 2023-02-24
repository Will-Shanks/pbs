/// Different op codes that can be set on an Attrl's value
use pbs_sys::batch_op;

#[derive(Debug)]
pub enum Op {
    Set(String),
    Unset(String),
    Incr(String),
    Decr(String),
    Equal(String),
    NotEqual(String),
    GreaterThan(String),
    LessThan(String),
    EqualOrGreaterThan(String),
    EqualOrLessThan(String),
    Default(String),
}

impl Op {
    pub fn op(&self) -> batch_op {
        match self {
            Op::Set(_) => batch_op::SET,
            Op::Unset(_) => batch_op::UNSET,
            Op::Incr(_) => batch_op::INCR,
            Op::Decr(_) => batch_op::DECR,
            Op::Equal(_) => batch_op::EQ,
            Op::NotEqual(_) => batch_op::NE,
            Op::GreaterThan(_) => batch_op::GT,
            Op::LessThan(_) => batch_op::LT,
            Op::EqualOrGreaterThan(_) => batch_op::GE,
            Op::EqualOrLessThan(_) => batch_op::LE,
            Op::Default(_) => batch_op::DFLT,
        }
    }
    pub fn val(&self) -> String {
        match self {
            Op::Set(x) => x.to_string(),
            Op::Unset(x) => x.to_string(),
            Op::Incr(x) => x.to_string(),
            Op::Decr(x) => x.to_string(),
            Op::Equal(x) => x.to_string(),
            Op::NotEqual(x) => x.to_string(),
            Op::GreaterThan(x) => x.to_string(),
            Op::LessThan(x) => x.to_string(),
            Op::EqualOrGreaterThan(x) => x.to_string(),
            Op::EqualOrLessThan(x) => x.to_string(),
            Op::Default(x) => x.to_string(),
        }
    }
}
