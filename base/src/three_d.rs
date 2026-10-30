//! IronXL: 3-D references, such as `Data1:Data3!A1` or `'My Jan:Mar'!A1:B2`.
//!
//! A 3-D reference names the same cell or range on each sheet from its first end to its last end, in the order of the
//! sheets. The ends can be in any order. As in Excel, only the functions of [`takes_three_d`] read it: each 3-D argument
//! becomes one reference for each sheet of the span, and the function reads them with its rules for a reference of one
//! sheet. A 3-D reference in every other place gives `#VALUE!`. An end that no sheet has gives `#REF!`.

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// True for each function that Excel lets take a 3-D reference.
pub(crate) fn takes_three_d(kind: &Function) -> bool {
    matches!(
        kind,
        Function::Sum
            | Function::Average
            | Function::Averagea
            | Function::Count
            | Function::Counta
            | Function::Max
            | Function::MaxA
            | Function::Min
            | Function::MinA
            | Function::Product
            | Function::Stdev
            | Function::Stdeva
            | Function::StDevP
            | Function::Stdevpa
            | Function::StDevS
            | Function::StDevPCompat
            | Function::VarA
            | Function::VarP
            | Function::VarpA
            | Function::VarS
            | Function::VarCompat
            | Function::VarPCompat
    )
}

/// The sheet indexes from the lower end to the higher end, or None when no sheet has the name of an end.
pub(crate) fn span(first_index: Option<u32>, last_index: Option<u32>) -> Option<Vec<u32>> {
    let (first, last) = (first_index?, last_index?);
    Some((first.min(last)..=first.max(last)).collect())
}

/// The range of a 3-D reference on the sheet `sheet` with the name `name`. A cell is a range of one cell, so each function
/// reads it with its rules for a range, as Excel reads a 3-D reference.
pub(crate) fn on_sheet(reference: &Node, sheet: u32, name: &str) -> Node {
    let sheet_name = Some(name.to_string());
    match reference {
        Node::ReferenceKind {
            absolute_row,
            absolute_column,
            row,
            column,
            ..
        } => Node::RangeKind {
            sheet_name,
            sheet_index: sheet,
            absolute_row1: *absolute_row,
            absolute_column1: *absolute_column,
            row1: *row,
            column1: *column,
            absolute_row2: *absolute_row,
            absolute_column2: *absolute_column,
            row2: *row,
            column2: *column,
        },
        Node::RangeKind {
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
            ..
        } => Node::RangeKind {
            sheet_name,
            sheet_index: sheet,
            absolute_row1: *absolute_row1,
            absolute_column1: *absolute_column1,
            row1: *row1,
            column1: *column1,
            absolute_row2: *absolute_row2,
            absolute_column2: *absolute_column2,
            row2: *row2,
            column2: *column2,
        },
        other => other.clone(),
    }
}

impl Model<'_> {
    /// The value of a 3-D reference outside a function that takes it: `#REF!` when an end is not a sheet, else `#VALUE!`.
    pub(crate) fn evaluate_three_d(
        &self,
        first_index: Option<u32>,
        last_index: Option<u32>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match span(first_index, last_index) {
            Some(_) => CalcResult::new_error(
                Error::VALUE,
                cell,
                "A 3-D reference is valid only in a function that takes it".to_string(),
            ),
            None => CalcResult::new_error(Error::REF, cell, "Sheet not found".to_string()),
        }
    }

    /// The arguments of a function that takes a 3-D reference, with each 3-D argument replaced by one reference for each
    /// sheet of its span, in the order of the sheets. None when an end of a 3-D argument is not a sheet.
    pub(crate) fn three_d_arguments(&self, args: &[Node]) -> Option<Vec<Node>> {
        let mut expanded = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                Node::Reference3DKind {
                    first_index,
                    last_index,
                    reference,
                    ..
                } => {
                    for sheet in span(*first_index, *last_index)? {
                        let name = &self.workbook.worksheets.get(sheet as usize)?.name;
                        expanded.push(on_sheet(reference, sheet, name));
                    }
                }
                other => expanded.push(other.clone()),
            }
        }
        Some(expanded)
    }
}
