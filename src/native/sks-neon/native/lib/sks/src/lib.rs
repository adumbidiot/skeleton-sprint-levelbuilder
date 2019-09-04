mod block;

pub use block::BackgroundType;
pub use block::Block;
pub use block::Direction;

use ratel::ast::expression::BinaryExpression;
use ratel::ast::expression::Expression;
use ratel::ast::Expression::Literal as ExprLiteral;
use ratel::ast::ExpressionNode;
use ratel::ast::Literal;
use ratel_visitor::Visitable;
use ratel_visitor::Visitor;

pub const LEVEL_WIDTH: usize = 32;
pub const LEVEL_HEIGHT: usize = 18;

impl Block {
    pub fn as_lbl(&self) -> &str {
        match self {
            Block::Block => "B0",
            Block::Empty => "00",
            Block::Exit => "E0",
            Block::Switch => "S0",
            Block::Player => "X0",
            Block::ToggleBlock { solid: true } => "T0",
            Block::ToggleBlock { solid: false } => "T1",
            Block::Torch => "D1",
            _ => unimplemented!(),
        }
    }
}

pub type As3Result<T> = Result<T, As3Error>;

#[derive(Debug)]
pub enum As3Error {
    InvalidLBL,
    InvalidLevelSize,
    Generic(&'static str),
}

struct LevelArrayVisitor {
    count: usize,
    data: Vec<Block>,
    error: Option<As3Error>,
}

impl LevelArrayVisitor {
    fn validate_left_expr(&mut self, expr: Expression) -> bool {
        if let Expression::ComputedMember(expr) = expr {
            if let Expression::ComputedMember(inner_expr) = expr.object.item {
                if let Expression::Identifier("lvlArray") = inner_expr.object.item {
                    if let ExprLiteral(Literal::Number(level)) = inner_expr.property.item {
                        if let ExprLiteral(Literal::Number(row)) = expr.property.item {
                            if row
                                .parse::<usize>()
                                .map(|el| el == self.count)
                                .unwrap_or(false)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn process_right(&mut self, expr: Expression) -> As3Result<()> {
        if let Expression::Array(expr) = expr {
            let mut i = 0;
            for node in expr.body.iter() {
                let data = match node.item {
                    Expression::Identifier(data) => data,
                    ExprLiteral(Literal::Number(data)) => data,
                    ExprLiteral(Literal::String(data)) => data.trim_matches('"'),
                    _ => return Err(As3Error::Generic("Unknown Item type")),
                };

                let block = Block::from_lbl(data).ok_or(As3Error::InvalidLBL)?;
                self.data.push(block);

                i += 1;
            }

            if i != 32 {
                return Err(As3Error::InvalidLevelSize);
            }

            self.count += 1;
        } else {
            return Err(As3Error::Generic("Invalid Expr Type"));
        }

        Ok(())
    }

    fn get_data(self) -> As3Result<Vec<Block>> {
        if let Some(e) = self.error {
            return Err(e);
        }

        if self.data.len() != LEVEL_WIDTH * LEVEL_HEIGHT {
            return Err(As3Error::InvalidLevelSize);
        }

        Ok(self.data)
    }

    fn new() -> Self {
        LevelArrayVisitor {
            count: 0,
            data: Vec::with_capacity(LEVEL_WIDTH * LEVEL_HEIGHT),
            error: None,
        }
    }
}

impl<'ast> Visitor<'ast> for LevelArrayVisitor {
    fn on_binary_expression(
        &mut self,
        item: &BinaryExpression<'ast>,
        _node: &'ast ExpressionNode<'ast>,
    ) {
        if self.error.is_some() {
            return;
        }

        if !self.validate_left_expr(**item.left) {
            self.error = Some(As3Error::Generic("Left Parse"));
        }

        if let Err(e) = self.process_right(**item.right) {
            self.error = Some(e);
        }
    }
}

pub fn decode_as3(data: &str) -> As3Result<Vec<Block>> {
    let ast = ratel::parse(data).map_err(|_| As3Error::Generic("Invalid Parse"))?;
    let mut visitor = LevelArrayVisitor::new();
    ast.visit_with(&mut visitor);
    visitor.get_data()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let file_data = std::fs::read_to_string("kitchen_sink_as3.txt").unwrap();
        let _data = decode_as3(&file_data).unwrap();
    }
}
