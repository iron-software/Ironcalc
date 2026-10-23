//! IronXL: evaluates the static precedents of a formula before the formula itself.
//!
//! `Model::evaluate_cell` evaluates the precedents of a formula recursively, so a dependency
//! chain of `n` links needs `n` nested calls and a long chain overflows the stack. Before a
//! cell is evaluated, the functions here walk its precedents with an explicit stack and
//! evaluate them in dependency order (post-order). When the cell is evaluated, its precedents
//! are already evaluated and the recursion stays shallow.
//!
//! The walk only changes the order of evaluation, never a value. It evaluates a precedent
//! early only when its value cannot depend on that order: every cell that it reads, directly
//! or through other cells, is known from the formulas, is not on a cycle and is not part of an
//! array formula that is not evaluated yet. The recursive evaluation handles every other cell
//! as before:
//! * a formula with references that only its evaluation finds (`INDIRECT`, `OFFSET`, the
//!   range operator, the spill range operator, a defined-name `LAMBDA`), and every cell that
//!   reads such a formula,
//! * a cell on a cycle, and every cell that reads it, so `#CIRC!` comes as before,
//! * an array formula or a spill cell whose anchor is not evaluated yet.

use std::collections::{BTreeSet, HashMap};

use crate::{
    expressions::{parser::Node, types::CellReferenceIndex},
    model::{CellState, Model, ParsedDefinedName},
    types::{Cell, SheetData},
};

/// A range with at most this many cells is read cell by cell. A larger range is read from
/// an index of the formula cells, so a whole-column range does not visit every row.
const SMALL_RANGE: i64 = 1024;

/// The walk keeps the precedents of at most this many parsed formulas. A pass over many distinct formulas, such as a
/// column with a formula of its own in each cell, then holds bounded memory.
const KEPT_FORMULAS: usize = 65_536;

/// A cell as `(sheet, row, column)`.
type CellKey = (u32, i32, i32);

/// A reference in a formula. A coordinate that is not absolute is relative to the formula cell.
#[derive(Clone, Copy)]
enum Precedent {
    Cell {
        sheet: u32,
        row: i32,
        column: i32,
        absolute_row: bool,
        absolute_column: bool,
    },
    Range {
        sheet: u32,
        row1: i32,
        column1: i32,
        absolute_row1: bool,
        absolute_column1: bool,
        row2: i32,
        column2: i32,
        absolute_row2: bool,
        absolute_column2: bool,
    },
}

/// The static references of a parsed formula.
#[derive(Default)]
struct FormulaPrecedents {
    /// The formula has references that only its evaluation finds.
    dynamic: bool,
    references: Vec<Precedent>,
}

enum VisitState {
    /// The cell is on the walk stack. A path back to it is a cycle.
    OnStack,
    /// The value of the cell can depend on the order of evaluation. The recursive
    /// evaluation evaluates it.
    Recursive,
}

/// A cell on the walk stack. Its precedents are `pending[start..end]`.
struct Frame {
    cell: CellKey,
    start: usize,
    next: usize,
    end: usize,
    /// The cell can not be evaluated early.
    recursive: bool,
}

/// What the walk does with a precedent.
enum Step {
    /// The precedent has a fixed value: nothing to do.
    Ready,
    /// The precedent makes the order of evaluation matter.
    Recursive,
    /// The precedent is a formula to walk.
    Visit,
}

/// The state of the precedent walks of one evaluation pass.
///
/// It is valid only while no formula changes, so a pass creates a new one.
#[derive(Default)]
pub(crate) struct PrecedentWalk {
    /// The precedents of each parsed formula, by `(sheet, formula index)`: at most `KEPT_FORMULAS` of them.
    formulas: HashMap<(u32, i32), FormulaPrecedents>,
    states: HashMap<CellKey, VisitState>,
    frames: Vec<Frame>,
    pending: Vec<CellKey>,
    /// The formula and spill cells of each sheet, built for the first large range.
    index: HashMap<u32, BTreeSet<(i32, i32)>>,
}

fn resolve(value: i32, absolute: bool, base: i32) -> i32 {
    if absolute {
        value
    } else {
        value + base
    }
}

fn is_formula_or_spill(cell: &Cell) -> bool {
    matches!(
        cell,
        Cell::CellFormula { .. } | Cell::ArrayFormula { .. } | Cell::SpillCell { .. }
    )
}

/// The formula and spill cells of a sheet, as `(column, row)`.
fn formula_cells_by_column(sheet_data: &SheetData) -> BTreeSet<(i32, i32)> {
    let mut cells = BTreeSet::new();
    for (&row, row_data) in sheet_data {
        for (&column, cell) in row_data {
            if is_formula_or_spill(cell) {
                cells.insert((column, row));
            }
        }
    }
    cells
}

impl Model<'_> {
    /// Evaluates a cell after its static precedents, so that the depth of the recursion does
    /// not grow with the length of a dependency chain (IronXL).
    pub(crate) fn evaluate_cell_after_precedents(
        &mut self,
        cell: CellReferenceIndex,
        walk: &mut PrecedentWalk,
    ) {
        self.evaluate_precedents(cell, walk);
        self.evaluate_cell(cell);
    }

    /// Evaluates the static precedents of `root` in dependency order, but not `root`.
    fn evaluate_precedents(&mut self, root: CellReferenceIndex, walk: &mut PrecedentWalk) {
        let root = (root.sheet, root.row, root.column);
        if self.cells.contains_key(&root) || walk.states.contains_key(&root) {
            return;
        }
        self.push_frame(root, walk);
        while let Some(top) = walk.frames.len().checked_sub(1) {
            let frame = &mut walk.frames[top];
            if frame.next < frame.end {
                let precedent = walk.pending[frame.next];
                frame.next += 1;
                match self.step(precedent, walk) {
                    Step::Ready => {}
                    Step::Recursive => walk.frames[top].recursive = true,
                    Step::Visit => self.push_frame(precedent, walk),
                }
                continue;
            }
            let Some(frame) = walk.frames.pop() else {
                break;
            };
            walk.pending.truncate(frame.start);
            if frame.recursive {
                walk.states.insert(frame.cell, VisitState::Recursive);
                if let Some(parent) = walk.frames.last_mut() {
                    parent.recursive = true;
                }
            } else {
                walk.states.remove(&frame.cell);
                if !walk.frames.is_empty() {
                    // Every precedent of the cell is evaluated, so this does not recurse deeply.
                    let (sheet, row, column) = frame.cell;
                    self.evaluate_cell(CellReferenceIndex { sheet, row, column });
                }
            }
        }
    }

    fn step(&self, cell: CellKey, walk: &PrecedentWalk) -> Step {
        if let Some(state) = self.cells.get(&cell) {
            return match state {
                CellState::Evaluated => Step::Ready,
                CellState::Evaluating => Step::Recursive,
            };
        }
        if walk.states.contains_key(&cell) {
            // On the stack (a cycle) or already found to need the recursive evaluation.
            return Step::Recursive;
        }
        match self.get_cell_at(cell) {
            Some(Cell::CellFormula { .. }) => Step::Visit,
            Some(Cell::ArrayFormula { .. }) => Step::Recursive,
            Some(Cell::SpillCell { a, .. }) => match self.cells.get(&(cell.0, a.0, a.1)) {
                Some(CellState::Evaluated) => Step::Ready,
                _ => Step::Recursive,
            },
            _ => Step::Ready,
        }
    }

    fn get_cell_at(&self, (sheet, row, column): CellKey) -> Option<&Cell> {
        self.workbook
            .worksheets
            .get(sheet as usize)?
            .sheet_data
            .get(&row)?
            .get(&column)
    }

    /// Puts a formula cell on the walk stack with the formula and spill cells that it reads.
    fn push_frame(&self, cell: CellKey, walk: &mut PrecedentWalk) {
        let formula = match self.get_cell_at(cell).and_then(Cell::get_formula) {
            Some(f) => f,
            None => return,
        };
        let (sheet, row, column) = cell;
        if walk.formulas.len() >= KEPT_FORMULAS && !walk.formulas.contains_key(&(sheet, formula)) {
            walk.formulas.clear();
        }
        let precedents = walk.formulas.entry((sheet, formula)).or_insert_with(|| {
            let mut precedents = FormulaPrecedents::default();
            if let Some((node, _)) = self
                .parsed_formulas
                .get(sheet as usize)
                .and_then(|formulas| formulas.get(formula as usize))
            {
                self.collect_precedents(node, &mut precedents);
            }
            precedents
        });
        let start = walk.pending.len();
        for precedent in &precedents.references {
            self.push_precedent_cells(
                *precedent,
                (row, column),
                &mut walk.index,
                &mut walk.pending,
            );
        }
        let recursive = precedents.dynamic;
        walk.frames.push(Frame {
            cell,
            start,
            next: start,
            end: walk.pending.len(),
            recursive,
        });
        walk.states.insert(cell, VisitState::OnStack);
    }

    /// Adds the formula and spill cells of a reference made from the cell at `base`.
    fn push_precedent_cells(
        &self,
        precedent: Precedent,
        base: (i32, i32),
        index: &mut HashMap<u32, BTreeSet<(i32, i32)>>,
        out: &mut Vec<CellKey>,
    ) {
        let (sheet, row1, column1, row2, column2) = match precedent {
            Precedent::Cell {
                sheet,
                row,
                column,
                absolute_row,
                absolute_column,
            } => {
                let row = resolve(row, absolute_row, base.0);
                let column = resolve(column, absolute_column, base.1);
                (sheet, row, column, row, column)
            }
            Precedent::Range {
                sheet,
                row1,
                column1,
                absolute_row1,
                absolute_column1,
                row2,
                column2,
                absolute_row2,
                absolute_column2,
            } => {
                let r1 = resolve(row1, absolute_row1, base.0);
                let r2 = resolve(row2, absolute_row2, base.0);
                let c1 = resolve(column1, absolute_column1, base.1);
                let c2 = resolve(column2, absolute_column2, base.1);
                (sheet, r1.min(r2), c1.min(c2), r1.max(r2), c1.max(c2))
            }
        };
        let Some(worksheet) = self.workbook.worksheets.get(sheet as usize) else {
            return;
        };
        let sheet_data = &worksheet.sheet_data;
        let height = i64::from(row2) - i64::from(row1) + 1;
        let width = i64::from(column2) - i64::from(column1) + 1;
        if height * width <= SMALL_RANGE {
            for row in row1..=row2 {
                if let Some(row_data) = sheet_data.get(&row) {
                    for column in column1..=column2 {
                        if row_data.get(&column).is_some_and(is_formula_or_spill) {
                            out.push((sheet, row, column));
                        }
                    }
                }
            }
            return;
        }
        let cells = index
            .entry(sheet)
            .or_insert_with(|| formula_cells_by_column(sheet_data));
        // Visit only the columns with formula cells, and in each column only the rows of the range.
        let mut column = column1;
        while let Some(&(next, _)) = cells.range((column, row1)..).next() {
            if next > column2 {
                break;
            }
            if next > column {
                column = next;
                continue;
            }
            for &(_, row) in cells.range((column, row1)..=(column, row2)) {
                out.push((sheet, row, column));
            }
            if column == column2 {
                break;
            }
            column += 1;
        }
    }

    /// Collects the static references of a formula.
    fn collect_precedents(&self, node: &Node, out: &mut FormulaPrecedents) {
        use Node::*;
        match node {
            ReferenceKind {
                sheet_index,
                absolute_row,
                absolute_column,
                row,
                column,
                ..
            } => out.references.push(Precedent::Cell {
                sheet: *sheet_index,
                row: *row,
                column: *column,
                absolute_row: *absolute_row,
                absolute_column: *absolute_column,
            }),
            RangeKind {
                sheet_index,
                absolute_row1,
                absolute_column1,
                row1,
                column1,
                absolute_row2,
                absolute_column2,
                row2,
                column2,
                ..
            } => out.references.push(Precedent::Range {
                sheet: *sheet_index,
                row1: *row1,
                column1: *column1,
                absolute_row1: *absolute_row1,
                absolute_column1: *absolute_column1,
                row2: *row2,
                column2: *column2,
                absolute_row2: *absolute_row2,
                absolute_column2: *absolute_column2,
            }),
            DefinedNameKind((name, scope, _)) => match self.get_parsed_defined_name(name, *scope) {
                Ok(Some(ParsedDefinedName::CellReference(reference))) => {
                    out.references.push(Precedent::Cell {
                        sheet: reference.sheet,
                        row: reference.row,
                        column: reference.column,
                        absolute_row: true,
                        absolute_column: true,
                    })
                }
                Ok(Some(ParsedDefinedName::RangeReference(range))) => {
                    out.references.push(Precedent::Range {
                        sheet: range.left.sheet,
                        row1: range.left.row,
                        column1: range.left.column,
                        absolute_row1: true,
                        absolute_column1: true,
                        row2: range.right.row,
                        column2: range.right.column,
                        absolute_row2: true,
                        absolute_column2: true,
                    })
                }
                Ok(Some(ParsedDefinedName::LambdaDefinition(..))) => out.dynamic = true,
                _ => {}
            },
            FunctionKind { kind, args } => {
                if kind.returns_reference() {
                    out.dynamic = true;
                }
                for arg in args {
                    self.collect_precedents(arg, out);
                }
            }
            NamedFunctionKind { id, args, .. } => {
                // Without an id, the function is a defined-name LAMBDA with a body of its own.
                if id.is_none() {
                    out.dynamic = true;
                }
                for arg in args {
                    self.collect_precedents(arg, out);
                }
            }
            OpRangeKind { left, right } => {
                out.dynamic = true;
                self.collect_precedents(left, out);
                self.collect_precedents(right, out);
            }
            SpillRangeOperator { child } => {
                out.dynamic = true;
                self.collect_precedents(child, out);
            }
            OpConcatenateKind { left, right }
            | OpSumKind { left, right, .. }
            | OpProductKind { left, right, .. }
            | OpPowerKind { left, right }
            | CompareKind { left, right, .. } => {
                self.collect_precedents(left, out);
                self.collect_precedents(right, out);
            }
            UnaryKind { right, .. } => self.collect_precedents(right, out),
            ImplicitIntersection { child, .. } => self.collect_precedents(child, out),
            LambdaDefKind { body, .. } => self.collect_precedents(body, out),
            LambdaCallKind { lambda, args } => {
                self.collect_precedents(lambda, out);
                for arg in args {
                    self.collect_precedents(arg, out);
                }
            }
            BooleanKind(_)
            | NumberKind(_)
            | StringKind(_)
            | WrongReferenceKind { .. }
            | WrongRangeKind { .. }
            | ArrayKind(_)
            | TableNameKind(_)
            | NamedVariableKind { .. }
            | ErrorKind(_)
            | ParseErrorKind { .. }
            | EmptyArgKind => {}
        }
    }
}
