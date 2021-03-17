use std::ops::Not;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Var(pub i32);

impl Var {
    pub fn literal(self) -> Literal {
        Literal::new(self, false)        
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Literal(pub i32);

impl Literal {
    pub fn new(Var(i): Var, negated: bool) -> Literal {
        Literal(if negated { -i } else { i })
    }

    pub fn var(self) -> Var {
        if self.0 < 0 {
            Var(-self.0)
        } else {
            Var(self.0)
        }
    }

    pub fn var_id(self) -> usize {
        self.var().0 as usize
    }

    pub fn is_negated(self) -> bool {
        self.0 < 0
    }

    pub fn value(self) -> Value {
        if self.0 < 0 {
            Value::False
        } else {
            Value::True
        }
    }
}

impl Not for Literal {
    type Output = Literal;

    fn not(self) -> Literal {
        Literal(-self.0)
    }
}

pub const VAR_INVALID: Var = Var(0);
pub const LIT_INVALID: Literal = Literal(0);

pub type Clause = Vec<Literal>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Value {
    Undet, True, False
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Value {
        match self {
            Value::Undet => Value::Undet,
            Value::True => Value::False,
            Value::False => Value::True,
        }
    }
}