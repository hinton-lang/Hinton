use crate::errors::ErrorReport;
use crate::parser::ast::ASTNodeIdx;
use crate::parser::Parser;

impl<'a> Parser<'a> {
  pub fn parse_stmt(&mut self) -> Result<ASTNodeIdx, ErrorReport> {
    self.parse_expr()
  }
}
