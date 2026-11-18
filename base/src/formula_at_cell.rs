//! IronXL: evaluate a formula text as if it is in a cell, and store nothing.
//!
//! A data validation rule of a spreadsheet holds a formula that is relative to the first cell of the
//! rule. A check of a value evaluates that formula at a cell of the rule, optionally with a test value
//! in that cell. The check must not change the workbook, so this module keeps every cell, every result,
//! and every evaluation state of the model as it was.

use std::mem;

use crate::{
    calc_result::CalcResult,
    cell::CellValue,
    expressions::{
        parser::ArrayNode,
        token::Error,
        types::{Area, CellReferenceIndex, CellReferenceRC},
    },
    model::Model,
    types::Cell,
};

/// IronXL: the result of [`Model::evaluate_formula_at`].
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaResult {
    /// The formula gives an empty cell.
    Empty,
    /// A number.
    Number(f64),
    /// A text.
    Text(String),
    /// A Boolean.
    Boolean(bool),
    /// An error value, such as `#DIV/0!`.
    Error(Error),
    /// A reference to the cells of an area, such as `=$A$1:$A$5` or a defined name of a range.
    Reference(Area),
}

impl Model<'_> {
    /// IronXL: evaluates `formula` as if it is in the cell `cell`, and stores nothing.
    ///
    /// * `formula` is the formula text, with or without the leading `=`, in the English grammar of a file.
    /// * `anchor` is the cell that the relative references of `formula` are relative to. When `cell` is
    ///   not `anchor`, each relative reference moves by the distance from `anchor` to `cell`, as a copy of
    ///   the formula moves it.
    /// * `value`, when given, is the value of `cell` during the evaluation. `CellValue::None` is an empty
    ///   cell. A formula that reads `cell` gets this value. A formula of another cell keeps its result.
    ///
    /// The evaluation reads the current result of each formula. It does not evaluate a formula of a cell
    /// again. After the call, each cell, each result, and each evaluation state is as before.
    ///
    /// A result that is an array gives its first element. A result that is a reference to one area gives
    /// [`FormulaResult::Reference`]. A formula that the engine cannot parse gives an error result.
    ///
    /// Returns an error, and changes nothing, when a sheet index is not a sheet of the workbook.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ironcalc_base::Model;
    /// # use ironcalc_base::cell::CellValue;
    /// # use ironcalc_base::expressions::types::CellReferenceIndex;
    /// # use ironcalc_base::FormulaResult;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new_empty("model", "en", "UTC", "en")?;
    /// model.set_user_input(0, 1, 1, "4".to_string())?;
    /// model.set_user_input(0, 2, 1, "5".to_string())?;
    /// model.evaluate();
    /// let a1 = CellReferenceIndex { sheet: 0, row: 1, column: 1 };
    /// let a2 = CellReferenceIndex { sheet: 0, row: 2, column: 1 };
    /// // The reference A1 moves to A2 at the cell A2.
    /// assert_eq!(model.evaluate_formula_at("=A1*2", a1, a2, None)?, FormulaResult::Number(10.0));
    /// // A test value in A1 replaces 4 during the evaluation only.
    /// let value = CellValue::Number(7.0);
    /// assert_eq!(model.evaluate_formula_at("=A1*2", a1, a1, Some(&value))?, FormulaResult::Number(14.0));
    /// assert_eq!(model.get_formatted_cell_value(0, 1, 1)?, "4");
    /// # Ok(())
    /// # }
    /// ```
    pub fn evaluate_formula_at(
        &mut self,
        formula: &str,
        anchor: CellReferenceIndex,
        cell: CellReferenceIndex,
        value: Option<&CellValue>,
    ) -> Result<FormulaResult, String> {
        let sheet_count = self.workbook.worksheets.len() as u32;
        if anchor.sheet >= sheet_count || cell.sheet >= sheet_count {
            return Err(format!(
                "Invalid sheet index: {}",
                anchor.sheet.max(cell.sheet)
            ));
        }
        let trimmed = formula.trim();
        let body = trimmed.strip_prefix('=').unwrap_or(trimmed);
        let context = CellReferenceRC {
            sheet: self.workbook.worksheets[anchor.sheet as usize].get_name(),
            row: anchor.row,
            column: anchor.column,
        };
        let node = self.parse_internal_formula(body, &context);
        // The parse gives each relative reference as a distance from `anchor`. The evaluation adds that
        // distance to the cell where it runs, so the references move from `anchor` to `cell`.
        let at = cell;

        let strings = self.workbook.shared_strings.len();
        let original = value.map(|value| self.put_test_value(at, value));
        let variables = mem::take(&mut self.variable_stack);
        let lambdas = mem::take(&mut self.lambdas);
        let support = self.support.remove(&at);
        let read_only = mem::replace(&mut self.read_only, true);

        let result = self.evaluate_node_in_context(&node, at);
        let result = self.formula_result(result);

        self.read_only = read_only;
        match support {
            Some(support) => {
                self.support.insert(at, support);
            }
            None => {
                self.support.remove(&at);
            }
        }
        self.lambdas = lambdas;
        self.variable_stack = variables;
        if let Some(original) = original {
            self.restore_cell(at, original);
        }
        for text in self.workbook.shared_strings.drain(strings..) {
            self.shared_strings.remove(&text);
        }
        Ok(result)
    }

    /// Puts `value` in the cell `at`, and gives the cell that was there before.
    fn put_test_value(&mut self, at: CellReferenceIndex, value: &CellValue) -> Option<Cell> {
        let cell = match value {
            CellValue::None => Cell::EmptyCell { s: 0 },
            CellValue::Number(number) => Cell::NumberCell { v: *number, s: 0 },
            CellValue::Boolean(flag) => Cell::BooleanCell { v: *flag, s: 0 },
            CellValue::String(text) => {
                let si = match self.shared_strings.get(text) {
                    Some(index) => *index,
                    None => {
                        let index = self.workbook.shared_strings.len();
                        self.workbook.shared_strings.push(text.clone());
                        self.shared_strings.insert(text.clone(), index);
                        index
                    }
                };
                Cell::SharedString {
                    si: si as i32,
                    s: 0,
                }
            }
        };
        self.workbook.worksheets[at.sheet as usize]
            .sheet_data
            .entry(at.row)
            .or_default()
            .insert(at.column, cell)
    }

    /// Puts back the cell that [`Model::put_test_value`] replaced. No cell before gives no cell.
    fn restore_cell(&mut self, at: CellReferenceIndex, original: Option<Cell>) {
        let sheet_data = &mut self.workbook.worksheets[at.sheet as usize].sheet_data;
        match original {
            Some(cell) => {
                sheet_data
                    .entry(at.row)
                    .or_default()
                    .insert(at.column, cell);
            }
            None => {
                if let Some(row) = sheet_data.get_mut(&at.row) {
                    row.remove(&at.column);
                    if row.is_empty() {
                        sheet_data.remove(&at.row);
                    }
                }
            }
        }
    }

    /// The public form of an evaluation result.
    fn formula_result(&mut self, result: CalcResult) -> FormulaResult {
        match result {
            CalcResult::String(text) => FormulaResult::Text(text),
            CalcResult::Number(number) => FormulaResult::Number(number),
            CalcResult::Boolean(flag) => FormulaResult::Boolean(flag),
            CalcResult::Error { error, .. } => FormulaResult::Error(error),
            CalcResult::EmptyCell | CalcResult::EmptyArg => FormulaResult::Empty,
            CalcResult::Range { left, right } => FormulaResult::Reference(Area {
                sheet: left.sheet,
                row: left.row.min(right.row),
                column: left.column.min(right.column),
                width: (right.column - left.column).abs() + 1,
                height: (right.row - left.row).abs() + 1,
            }),
            CalcResult::Array(rows) => match rows.first().and_then(|row| row.first()) {
                Some(ArrayNode::Number(number)) => FormulaResult::Number(*number),
                Some(ArrayNode::String(text)) => FormulaResult::Text(text.clone()),
                Some(ArrayNode::Boolean(flag)) => FormulaResult::Boolean(*flag),
                Some(ArrayNode::Error(error)) => FormulaResult::Error(error.clone()),
                Some(ArrayNode::Empty) | None => FormulaResult::Empty,
            },
            CalcResult::Lambda(_) => FormulaResult::Error(Error::CALC),
        }
    }
}
