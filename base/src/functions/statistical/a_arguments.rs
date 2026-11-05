//! The argument rule of the A functions: `AVERAGEA`, `MAXA`, `MINA`, `STDEVA`, `STDEVPA`,
//! `VARA`, and `VARPA`.
//!
//! - A reference argument (a range, one cell, or a defined name of one cell) counts a number,
//!   counts text as 0, `TRUE` as 1 and `FALSE` as 0, and skips an empty cell.
//!   A one-cell reference gets the same rule as a range, as in Excel.
//! - A direct argument counts a number and a Boolean. Direct text that is a number counts;
//!   other direct text gives `#VALUE!`.
//! - An error gives that error.

use crate::calc_result::CalcResult;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::model::{CellOrRange, Model, ParsedDefinedName};

/// How an A function reads the values of an array argument.
#[derive(Clone, Copy)]
pub(crate) enum ArrayValues {
    /// A number counts, a Boolean counts as 1 or 0, and text counts as 0.
    TextAsZero,
    /// Only a number counts.
    NumbersOnly,
    /// An array gives `#N/IMPL!`.
    Unsupported,
}

impl<'a> Model<'a> {
    /// Calls `f` with each value of `args` under the A-function rule.
    pub(crate) fn for_each_value_a<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        arrays: ArrayValues,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        for arg in args {
            match self.evaluate_a_argument(arg, cell) {
                CalcResult::Number(value) => f(value),
                CalcResult::Boolean(value) => f(if value { 1.0 } else { 0.0 }),
                CalcResult::String(value) => match self.cast_number(&value) {
                    Some(parsed) => f(parsed),
                    None => {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Argument cannot be cast into number".to_string(),
                        ))
                    }
                },
                CalcResult::Range { left, right } => {
                    self.for_each_range_value_a(left, right, cell, &mut f)?
                }
                CalcResult::Array(array) => for_each_array_value_a(array, arrays, cell, &mut f)?,
                error @ CalcResult::Error { .. } => return Err(error),
                CalcResult::Lambda(_) if matches!(arrays, ArrayValues::Unsupported) => {
                    return Err(not_implemented(cell))
                }
                CalcResult::Lambda(_) | CalcResult::EmptyCell | CalcResult::EmptyArg => {}
            }
        }
        Ok(())
    }

    /// Evaluates an argument of an A function. A one-cell reference and a defined name of one
    /// cell, absolute or relative, give a range of one cell, so the range rule reads them.
    fn evaluate_a_argument(&mut self, arg: &Node, cell: CellReferenceIndex) -> CalcResult {
        match arg {
            Node::ReferenceKind { .. } => {
                let range = self.evaluate_node_with_reference(arg, cell);
                if let CalcResult::Range { left, .. } = range {
                    // A cell read records its support, as `evaluate_node_in_context` does.
                    self.support
                        .entry(cell)
                        .or_default()
                        .push(CellOrRange::Cell((left.sheet, left.row, left.column)));
                }
                range
            }
            Node::DefinedNameKind((name, scope, _))
            | Node::SheetDefinedNameKind {
                name,
                sheet_index: scope @ Some(_),
                formula: Some(_),
                ..
            } => match self.get_parsed_defined_name(name, *scope) {
                Ok(Some(ParsedDefinedName::CellReference(reference))) => CalcResult::Range {
                    left: reference,
                    right: reference,
                },
                // A name with a relative part, such as `Sheet1!B$1`, is a one-cell reference at
                // the calling cell.
                Ok(Some(ParsedDefinedName::Formula(node @ Node::ReferenceKind { .. }))) => {
                    self.evaluate_a_argument(&node, cell)
                }
                _ => self.evaluate_node_in_context(arg, cell),
            },
            _ => self.evaluate_node_in_context(arg, cell),
        }
    }

    /// Calls `f` with each value of the range from `left` to `right`. Text counts as 0, a
    /// Boolean as 1 or 0, and an empty cell is skipped. A whole row or column stops at the
    /// used area of the sheet.
    fn for_each_range_value_a<F>(
        &mut self,
        left: CellReferenceIndex,
        right: CellReferenceIndex,
        cell: CellReferenceIndex,
        f: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        if left.sheet != right.sheet {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Ranges are in different sheets".to_string(),
            ));
        }
        let mut last_row = right.row;
        let mut last_column = right.column;
        if (left.row == 1 && right.row == LAST_ROW)
            || (left.column == 1 && right.column == LAST_COLUMN)
        {
            let dimension = match self.workbook.worksheet(left.sheet) {
                Ok(sheet) => sheet.dimension(),
                Err(_) => {
                    return Err(CalcResult::new_error(
                        Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", left.sheet),
                    ))
                }
            };
            if left.row == 1 && right.row == LAST_ROW {
                last_row = dimension.max_row;
            }
            if left.column == 1 && right.column == LAST_COLUMN {
                last_column = dimension.max_column;
            }
        }
        for row in left.row..=last_row {
            for column in left.column..=last_column {
                match self.evaluate_cell(CellReferenceIndex {
                    sheet: left.sheet,
                    row,
                    column,
                }) {
                    CalcResult::Number(value) => f(value),
                    CalcResult::Boolean(value) => f(if value { 1.0 } else { 0.0 }),
                    CalcResult::String(_) => f(0.0),
                    error @ CalcResult::Error { .. } => return Err(error),
                    CalcResult::Range { .. } => {
                        return Err(CalcResult::new_error(
                            Error::ERROR,
                            cell,
                            "Unexpected Range".to_string(),
                        ))
                    }
                    CalcResult::EmptyCell
                    | CalcResult::EmptyArg
                    | CalcResult::Array(_)
                    | CalcResult::Lambda(_) => {}
                }
            }
        }
        Ok(())
    }
}

/// Calls `f` with each value of an array argument under the rule `arrays`.
fn for_each_array_value_a<F>(
    array: Vec<Vec<ArrayNode>>,
    arrays: ArrayValues,
    cell: CellReferenceIndex,
    f: &mut F,
) -> Result<(), CalcResult>
where
    F: FnMut(f64),
{
    if matches!(arrays, ArrayValues::Unsupported) {
        return Err(not_implemented(cell));
    }
    for value in array.into_iter().flatten() {
        match value {
            ArrayNode::Number(value) => f(value),
            ArrayNode::Error(error) => {
                return Err(CalcResult::Error {
                    error,
                    origin: cell,
                    message: "Error in array".to_string(),
                })
            }
            ArrayNode::Boolean(value) if matches!(arrays, ArrayValues::TextAsZero) => {
                f(if value { 1.0 } else { 0.0 })
            }
            ArrayNode::String(_) if matches!(arrays, ArrayValues::TextAsZero) => f(0.0),
            ArrayNode::Boolean(_) | ArrayNode::String(_) | ArrayNode::Empty => {}
        }
    }
    Ok(())
}

fn not_implemented(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::Error {
        error: Error::NIMPL,
        origin: cell,
        message: "Arrays not supported yet".to_string(),
    }
}
