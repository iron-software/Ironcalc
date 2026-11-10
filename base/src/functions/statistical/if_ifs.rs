use crate::arithmetic::bcast_idx;
use crate::cast::calc_result_to_array_node;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::build_criteria_with_date_system;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    model::Model,
};

/// A compiled criterion predicate, as returned by `build_criteria`.
type Criterion<'c> = Box<dyn Fn(&CalcResult) -> bool + 'c>;

/// Converts a single array element into the equivalent scalar `CalcResult`,
/// used to feed `build_criteria` from an inline-array criteria argument.
fn array_node_to_calc_result(node: &ArrayNode, cell: CellReferenceIndex) -> CalcResult {
    match node {
        ArrayNode::Number(n) => CalcResult::Number(*n),
        ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
        ArrayNode::String(s) => CalcResult::String(s.clone()),
        ArrayNode::Error(e) => CalcResult::Error {
            error: e.clone(),
            origin: cell,
            message: "".to_string(),
        },
        ArrayNode::Empty => CalcResult::EmptyCell,
    }
}

/// IronXL: a criteria argument. A range or an array gives a grid with one criterion for each cell
/// or element.
enum CriteriaArg {
    Scalar(CalcResult),
    Grid(Vec<Vec<CalcResult>>),
}

impl CriteriaArg {
    /// The rows and columns of the criteria. A scalar has one row and one column.
    fn dims(&self) -> (usize, usize) {
        match self {
            CriteriaArg::Scalar(_) => (1, 1),
            CriteriaArg::Grid(grid) => (grid.len(), grid.first().map_or(0, Vec::len)),
        }
    }

    /// The criterion at `row` and `column` of the broadcast shape. A scalar or a length-1 axis
    /// repeats. A position that the grid does not have gives `None`.
    fn at(&self, row: usize, column: usize) -> Option<&CalcResult> {
        match self {
            CriteriaArg::Scalar(value) => Some(value),
            CriteriaArg::Grid(grid) => bcast_idx(grid.len(), row)
                .and_then(|r| grid.get(r))
                .and_then(|cells| bcast_idx(cells.len(), column).and_then(|c| cells.get(c))),
        }
    }
}

/// IronXL: gives true when both ranges have the same number of rows and columns. The IFS
/// functions compare the declared bounds, before the cut to the used range, as Excel does.
fn same_size(first: &Range, second: &Range) -> bool {
    first.right.row - first.left.row == second.right.row - second.left.row
        && first.right.column - first.left.column == second.right.column - second.left.column
}

/// IronXL: the sum range of SUMIF or the average range of AVERAGEIF, resized as Excel does. The
/// range starts at the top-left cell of `sum_range` and has the rows and columns of
/// `criteria_range`. It stops at the last row and the last column of the sheet.
fn resized_range(sum_range: &Range, criteria_range: &Range) -> Range {
    let mut right = sum_range.left;
    right.row = (right.row + criteria_range.right.row - criteria_range.left.row).min(LAST_ROW);
    right.column =
        (right.column + criteria_range.right.column - criteria_range.left.column).min(LAST_COLUMN);
    Range {
        left: sum_range.left,
        right,
    }
}

/// IronXL: the `#VALUE!` error of a criteria range with another size than the first range.
fn size_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "Ranges are of different sizes".to_string(),
    )
}

impl<'a> Model<'a> {
    /// COUNTIF(range, criteria). IronXL: a range or an array criterion gives one count for each
    /// criterion, in the shape of the criteria. A scalar criterion gives one count.
    pub(crate) fn fn_countif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        self.fn_countifs(args, cell)
    }

    /// AVERAGEIF(criteria_range, criteria, [average_range])
    /// if average_rage is missing then criteria_range will be used
    pub(crate) fn fn_averageif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() == 2 {
            let arguments = vec![args[0].clone(), args[0].clone(), args[1].clone()];
            self.average_ifs(&arguments, cell, false)
        } else if args.len() == 3 {
            // IronXL: AVERAGEIF gives no size error for an average range of another size.
            let arguments = vec![args[2].clone(), args[0].clone(), args[1].clone()];
            self.average_ifs(&arguments, cell, false)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    /// COUNTIFS(criteria_range1, criteria1, [criteria_range2, criteria2], ...). IronXL: a range or
    /// an array criterion gives one count for each criterion (see [`Model::broadcast_criteria`]).
    pub(crate) fn fn_countifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let args_count = args.len();
        if args_count < 2 || !args_count.is_multiple_of(2) {
            return CalcResult::new_args_number_error(cell);
        }
        let (ranges, criteria) = match self.criteria_pairs(args, cell) {
            Ok(pairs) => pairs,
            Err(error) => return error,
        };
        // IronXL: each criteria range must have the rows and columns of the first range.
        if ranges.iter().any(|range| !same_size(range, &ranges[0])) {
            return size_error(cell);
        }
        self.broadcast_criteria(&criteria, |model, fn_criteria| {
            match model.count_ifs(&ranges, fn_criteria, cell) {
                Ok(total) => CalcResult::Number(total),
                Err(error) => error,
            }
        })
    }

    /// IronXL: resolves each pair of a criteria range and a criteria argument in `pairs`. The
    /// criteria comes before its range, as before. An error of a range is the whole result.
    fn criteria_pairs(
        &mut self,
        pairs: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(Vec<Range>, Vec<CriteriaArg>), CalcResult> {
        let mut ranges = Vec::with_capacity(pairs.len() / 2);
        let mut criteria = Vec::with_capacity(pairs.len() / 2);
        for [range, criterion] in pairs.as_chunks::<2>().0 {
            criteria.push(self.criteria_arg(criterion, cell)?);
            ranges.push(self.node_to_range(range, cell)?);
        }
        Ok((ranges, criteria))
    }

    /// IronXL: calls `compute` with the predicates of the criteria. With scalar criteria only, the
    /// result of `compute` is the result. Else the result is an array with the largest rows and
    /// columns of the criteria, and `compute` runs once for each position. A scalar or a length-1
    /// axis repeats, and a position that one criteria array does not have gives `#N/A`, as in
    /// Excel. Shared by all criteria functions.
    fn broadcast_criteria<F>(&mut self, criteria: &[CriteriaArg], mut compute: F) -> CalcResult
    where
        F: FnMut(&mut Self, &[Criterion<'_>]) -> CalcResult,
    {
        let locale = self.locale;
        let date_system = self.date_system;
        let build = |row: usize, column: usize| -> Option<Vec<Criterion<'_>>> {
            criteria
                .iter()
                .map(|criterion| {
                    criterion
                        .at(row, column)
                        .map(|value| build_criteria_with_date_system(value, locale, date_system))
                })
                .collect()
        };
        if criteria
            .iter()
            .all(|criterion| matches!(criterion, CriteriaArg::Scalar(_)))
        {
            // A scalar criterion is at each position.
            let fn_criteria = build(0, 0).unwrap_or_default();
            return compute(self, &fn_criteria);
        }
        let (rows, columns) = criteria
            .iter()
            .map(CriteriaArg::dims)
            .fold((1, 1), |(rows, columns), (r, c)| {
                (rows.max(r), columns.max(c))
            });
        let mut output: Vec<Vec<ArrayNode>> = Vec::with_capacity(rows);
        for row in 0..rows {
            let mut out_row: Vec<ArrayNode> = Vec::with_capacity(columns);
            for column in 0..columns {
                out_row.push(match build(row, column) {
                    Some(fn_criteria) => calc_result_to_array_node(compute(self, &fn_criteria)),
                    None => ArrayNode::Error(Error::NA),
                });
            }
            output.push(out_row);
        }
        CalcResult::Array(output)
    }

    /// Counts the cells of the first range where the cell of each range at the same offset meets
    /// its criterion. `ranges` and `fn_criteria` are parallel. IronXL: shared by COUNTIFS and the
    /// array criteria path of COUNTIF.
    fn count_ifs(
        &mut self,
        ranges: &[Range],
        fn_criteria: &[Criterion<'_>],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let case_count = ranges.len();
        let mut total = 0.0;
        let first_range = &ranges[0];
        let left_row = first_range.left.row;
        let left_column = first_range.left.column;
        let right_row = first_range.right.row;
        let right_column = first_range.right.column;

        // IronXL: each criteria range reads the used range of its own sheet. A row or column past
        // the used range of every sheet is empty in each range.
        let mut max_row = 1;
        let mut max_column = 1;
        for range in ranges.iter() {
            let dimension = match self.workbook.worksheet(range.left.sheet) {
                Ok(s) => s.dimension(),
                Err(_) => {
                    return Err(CalcResult::new_error(
                        Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", range.left.sheet),
                    ))
                }
            };
            max_row = max_row.max(dimension.max_row);
            max_column = max_column.max(dimension.max_column);
        }

        let open_row = left_row == 1 && right_row == LAST_ROW;
        let open_column = left_column == 1 && right_column == LAST_COLUMN;

        for row in left_row..right_row + 1 {
            if open_row && row > max_row {
                // If the row is larger than the max row in the sheet then all cells are empty.
                // We compute it only once
                let mut is_true = true;
                for fn_criterion in fn_criteria.iter() {
                    if !fn_criterion(&CalcResult::EmptyCell) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    // IronXL: a whole sheet holds 17,179,869,184 cells; an i32 product wraps.
                    total +=
                        f64::from(LAST_ROW - max_row) * f64::from(right_column - left_column + 1);
                }
                break;
            }
            for column in left_column..right_column + 1 {
                if open_column && column > max_column {
                    // If the column is larger than the max column in the sheet then all cells are empty.
                    // We compute it only once
                    let mut is_true = true;
                    for fn_criterion in fn_criteria.iter() {
                        if !fn_criterion(&CalcResult::EmptyCell) {
                            is_true = false;
                            break;
                        }
                    }
                    if is_true {
                        total += f64::from(LAST_COLUMN - max_column);
                    }
                    break;
                }
                let mut is_true = true;
                for case_index in 0..case_count {
                    // We check if value in range n meets criterion n
                    let range = &ranges[case_index];
                    let fn_criterion = &fn_criteria[case_index];
                    let value = self.evaluate_cell(CellReferenceIndex {
                        sheet: range.left.sheet,
                        row: range.left.row + row - first_range.left.row,
                        column: range.left.column + column - first_range.left.column,
                    });
                    if !fn_criterion(&value) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    total += 1.0;
                }
            }
        }
        Ok(total)
    }

    /// Folds the matching values of SUMIFS, AVERAGEIFS, MINIFS, or MAXIFS. `fold` gets a `run`
    /// function that applies its argument to each matching value, and gives the result of one set of
    /// criteria. IronXL: a range or an array criterion gives one result for each criterion (see
    /// [`Model::broadcast_criteria`]). With `check_size`, a criteria range with another size than
    /// the sum range gives `#VALUE!`. SUMIF and AVERAGEIF call it without the check, as Excel gives
    /// no size error, and their sum range takes the size of their one criteria range.
    pub(crate) fn apply_ifs<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        check_size: bool,
        mut fold: F,
    ) -> CalcResult
    where
        F: FnMut(&mut dyn FnMut(&mut dyn FnMut(f64)) -> Result<(), CalcResult>) -> CalcResult,
    {
        let args_count = args.len();
        if args_count < 3 || args_count.is_multiple_of(2) {
            return CalcResult::new_args_number_error(cell);
        }
        let sum_range = match self.node_to_range(&args[0], cell) {
            Ok(range) => range,
            Err(error) => return error,
        };
        let (ranges, criteria) = match self.criteria_pairs(&args[1..], cell) {
            Ok(pairs) => pairs,
            Err(error) => return error,
        };
        let sum_range = if check_size {
            if ranges.iter().any(|range| !same_size(range, &sum_range)) {
                return size_error(cell);
            }
            sum_range
        } else {
            resized_range(&sum_range, &ranges[0])
        };
        self.broadcast_criteria(&criteria, |model, fn_criteria| {
            fold(&mut |apply| model.run_ifs(&sum_range, &ranges, fn_criteria, cell, apply))
        })
    }

    /// Walks `sum_range` and applies `apply` to every numeric cell whose parallel
    /// cell in each criteria range satisfies the matching criterion. `ranges` and
    /// `fn_criteria` are parallel (one criteria range and one predicate per case).
    pub(crate) fn run_ifs<F>(
        &mut self,
        sum_range: &Range,
        ranges: &[Range],
        fn_criteria: &[Criterion<'_>],
        cell: CellReferenceIndex,
        mut apply: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        let left_row = sum_range.left.row;
        let left_column = sum_range.left.column;
        let mut right_row = sum_range.right.row;
        let mut right_column = sum_range.right.column;

        // IronXL: a range that ends at the last row or column stops at the used range of its
        // sheet, also when it does not start at row 1 or column 1. Those cells hold no number.
        if right_row == LAST_ROW || right_column == LAST_COLUMN {
            let dimension = match self.workbook.worksheet(sum_range.left.sheet) {
                Ok(s) => s.dimension(),
                Err(_) => {
                    return Err(CalcResult::new_error(
                        Error::ERROR,
                        cell,
                        format!("Invalid worksheet index: '{}'", sum_range.left.sheet),
                    ));
                }
            };
            if right_row == LAST_ROW {
                right_row = dimension.max_row;
            }
            if right_column == LAST_COLUMN {
                right_column = dimension.max_column;
            }
        }

        for row in left_row..right_row + 1 {
            for column in left_column..right_column + 1 {
                let mut is_true = true;
                for (range, fn_criterion) in ranges.iter().zip(fn_criteria.iter()) {
                    // We check if value in range n meets criterion n
                    let value = self.evaluate_cell(CellReferenceIndex {
                        sheet: range.left.sheet,
                        row: range.left.row + row - sum_range.left.row,
                        column: range.left.column + column - sum_range.left.column,
                    });
                    if !fn_criterion(&value) {
                        is_true = false;
                        break;
                    }
                }
                if is_true {
                    let v = self.evaluate_cell(CellReferenceIndex {
                        sheet: sum_range.left.sheet,
                        row,
                        column,
                    });
                    match v {
                        CalcResult::Number(n) => apply(n),
                        CalcResult::Error { .. } => return Err(v),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// IronXL: evaluates a criteria argument. A range or an array gives a grid of scalar criteria,
    /// one for each cell or element.
    fn criteria_arg(
        &mut self,
        criteria: &Node,
        cell: CellReferenceIndex,
    ) -> Result<CriteriaArg, CalcResult> {
        match self.evaluate_node_in_context(criteria, cell) {
            CalcResult::Range { left, right } => {
                if left.sheet != right.sheet {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Ranges are in different sheets".to_string(),
                    ));
                }
                let mut grid = Vec::new();
                for r in left.row..=right.row {
                    let mut row = Vec::new();
                    for c in left.column..=right.column {
                        row.push(self.evaluate_cell(CellReferenceIndex {
                            sheet: left.sheet,
                            row: r,
                            column: c,
                        }));
                    }
                    grid.push(row);
                }
                Ok(CriteriaArg::Grid(grid))
            }
            CalcResult::Array(array) => Ok(CriteriaArg::Grid(
                array
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|node| array_node_to_calc_result(node, cell))
                            .collect()
                    })
                    .collect(),
            )),
            value => Ok(CriteriaArg::Scalar(value)),
        }
    }

    /// Evaluates `node` and requires it to be a single-sheet range.
    fn node_to_range(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        let value = self.evaluate_lookup_vector_node(node, cell);
        if value.is_error() {
            return Err(value);
        }
        if let CalcResult::Range { left, right } = value {
            if left.sheet != right.sheet {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Ranges are in different sheets".to_string(),
                ));
            }
            Ok(Range { left, right })
        } else {
            Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected a range".to_string(),
            ))
        }
    }

    pub(crate) fn fn_averageifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.average_ifs(args, cell, true)
    }

    /// AVERAGEIFS, and AVERAGEIF without the size check of `check_size`.
    fn average_ifs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        check_size: bool,
    ) -> CalcResult {
        self.apply_ifs(args, cell, check_size, |run| {
            let mut total = 0.0;
            let mut count = 0.0;
            if let Err(e) = run(&mut |value| {
                total += value;
                count += 1.0;
            }) {
                return e;
            }
            if count == 0.0 {
                return CalcResult::Error {
                    error: Error::DIV,
                    origin: cell,
                    message: "division by 0".to_string(),
                };
            }
            CalcResult::Number(total / count)
        })
    }

    pub(crate) fn fn_minifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_ifs(args, cell, true, |run| {
            let mut min = f64::INFINITY;
            if let Err(e) = run(&mut |value| min = value.min(min)) {
                return e;
            }
            CalcResult::Number(if min.is_infinite() { 0.0 } else { min })
        })
    }

    pub(crate) fn fn_maxifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.apply_ifs(args, cell, true, |run| {
            let mut max = -f64::INFINITY;
            if let Err(e) = run(&mut |value| max = value.max(max)) {
                return e;
            }
            CalcResult::Number(if max.is_infinite() { 0.0 } else { max })
        })
    }
}
