use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::worksheet::WorksheetDimension;
use crate::{
    calc_result::CalcResult,
    expressions::parser::Node,
    expressions::token::Error,
    model::{CellOrRange, Model},
};

use super::{
    binary_search::{
        binary_search_descending_or_greater, binary_search_descending_or_smaller,
        binary_search_or_greater, binary_search_or_smaller,
    },
    lookup_and_reference::array_node_to_calc_result,
    util::{compare_values, from_wildcard_to_regex, result_matches_regex},
};

#[derive(PartialEq)]
enum SearchMode {
    StartAtFirstItem = 1,
    StartAtLastItem = -1,
    BinarySearchDescending = -2,
    BinarySearchAscending = 2,
}

#[derive(PartialEq)]
enum MatchMode {
    ExactMatchSmaller = -1,
    ExactMatch = 0,
    ExactMatchLarger = 1,
    WildcardMatch = 2,
}

// lookup_value in array, match_mode search_mode
fn linear_search(
    lookup_value: &CalcResult,
    array: &[CalcResult],
    search_mode: SearchMode,
    match_mode: MatchMode,
) -> Option<usize> {
    let length = array.len();

    match match_mode {
        MatchMode::ExactMatch => {
            // exact match
            for l in 0..length {
                let index = if search_mode == SearchMode::StartAtFirstItem {
                    l
                } else {
                    length - l - 1
                };

                let value = &array[index];
                if compare_values(value, lookup_value) == 0 {
                    return Some(index);
                }
            }
            return None;
        }
        MatchMode::ExactMatchSmaller | MatchMode::ExactMatchLarger => {
            // exact match, if none found return the next smaller/larger item
            let mut found_index = 0;
            let mut approx = None;
            let m_mode = match_mode as i32;
            for l in 0..length {
                let index = if search_mode == SearchMode::StartAtFirstItem {
                    l
                } else {
                    length - l - 1
                };

                let value = &array[index];
                let c = compare_values(value, lookup_value);
                if c == 0 {
                    return Some(index);
                } else if c == m_mode {
                    match approx {
                        None => {
                            approx = Some(value.clone());
                            found_index = index;
                        }
                        Some(ref p) => {
                            if compare_values(p, value) == m_mode {
                                approx = Some(value.clone());
                                found_index = index;
                            }
                        }
                    }
                }
            }
            if approx.is_none() {
                return None;
            } else {
                return Some(found_index);
            }
        }
        MatchMode::WildcardMatch => {
            let result_matches: Box<dyn Fn(&CalcResult) -> bool> =
                if let CalcResult::String(s) = &lookup_value {
                    if let Ok(reg) = from_wildcard_to_regex(&s.to_lowercase(), true) {
                        Box::new(move |x| result_matches_regex(x, &reg))
                    } else {
                        Box::new(move |_| false)
                    }
                } else {
                    Box::new(move |x| compare_values(x, lookup_value) == 0)
                };
            for l in 0..length {
                let index = if search_mode == SearchMode::StartAtFirstItem {
                    l
                } else {
                    length - l - 1
                };
                let value = &array[index];
                if result_matches(value) {
                    return Some(index);
                }
            }
        }
    }
    None
}

/// A row or a column that XLOOKUP reads: a range reference or an array value.
enum LookupVector {
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
    Array(Vec<Vec<ArrayNode>>),
}

impl LookupVector {
    /// The number of rows and the number of columns, before a whole column or row is clamped.
    fn shape(&self) -> (i32, i32) {
        match self {
            LookupVector::Range { left, right } => {
                ((right.row - left.row) + 1, (right.column - left.column) + 1)
            }
            LookupVector::Array(rows) => array_shape(rows),
        }
    }
}

fn array_shape(rows: &[Vec<ArrayNode>]) -> (i32, i32) {
    (
        rows.len() as i32,
        rows.first().map(|row| row.len()).unwrap_or(0) as i32,
    )
}

impl<'a> Model<'a> {
    /// The XLOOKUP function searches a range or an array, and then returns the item corresponding
    /// to the first match it finds. If no match exists, then XLOOKUP can return the closest (approximate) match.
    /// =XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])
    ///
    /// lookup_array and return_array must be column or row arrays and of the same dimension.
    /// Otherwise #VALUE! is returned
    /// [if_not_found]
    /// Where a valid match is not found, return the [if_not_found] text you supply.
    /// If a valid match is not found, and [if_not_found] is missing, #N/A is returned.
    ///
    /// [match_mode]
    /// Specify the match type:
    ///   *  0 - Exact match. If none found, return #N/A. This is the default.
    ///   * -1 - Exact match. If none found, return the next smaller item.
    ///   *  1 - Exact match. If none found, return the next larger item.
    ///   *  2 - A wildcard match where *, ?, and ~ have special meaning.
    ///
    /// [search_mode]
    /// Specify the search mode to use:
    ///   *  1 - Perform a search starting at the first item. This is the default.
    ///   * -1 - Perform a reverse search starting at the last item.
    ///   *  2 - Perform a binary search that relies on lookup_array being sorted
    ///      in ascending order. If not sorted, invalid results will be returned.
    ///   * -2 - Perform a binary search that relies on lookup_array being sorted
    ///     in descending order. If not sorted, invalid results will be returned.
    pub(crate) fn fn_xlookup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 3 || args.len() > 6 {
            return CalcResult::new_args_number_error(cell);
        }
        let lookup_value = self.evaluate_node_in_context(&args[0], cell);
        if lookup_value.is_error() {
            return lookup_value;
        }
        // Get optional arguments
        let if_not_found = if args.len() >= 4 {
            let v = self.evaluate_node_in_context(&args[3], cell);
            match v {
                CalcResult::EmptyArg => CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Not found".to_string(),
                },
                _ => v,
            }
        } else {
            // default
            CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Not found".to_string(),
            }
        };
        let match_mode = if args.len() >= 5 {
            match self.get_number(&args[4], cell) {
                Ok(c) => match c.floor() as i32 {
                    -1 => MatchMode::ExactMatchSmaller,
                    1 => MatchMode::ExactMatchLarger,
                    0 => MatchMode::ExactMatch,
                    2 => MatchMode::WildcardMatch,
                    _ => {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Unexpected number".to_string(),
                        };
                    }
                },
                Err(s) => return s,
            }
        } else {
            // default
            MatchMode::ExactMatch
        };
        let search_mode = if args.len() == 6 {
            match self.get_number(&args[5], cell) {
                Ok(c) => match c.floor() as i32 {
                    1 => SearchMode::StartAtFirstItem,
                    -1 => SearchMode::StartAtLastItem,
                    -2 => SearchMode::BinarySearchDescending,
                    2 => SearchMode::BinarySearchAscending,
                    _ => {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Unexpected number".to_string(),
                        };
                    }
                },
                Err(s) => return s,
            }
        } else {
            // default
            SearchMode::StartAtFirstItem
        };
        // lookup_array: a range or an array value, as one row or one column
        let lookup = match self.evaluate_lookup_vector_node(&args[1], cell) {
            CalcResult::Range { left, right } => {
                if left.row != right.row && left.column != right.column {
                    // second argument must be a vector
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    };
                }
                LookupVector::Range { left, right }
            }
            CalcResult::Array(rows) => {
                let shape = array_shape(&rows);
                if shape.0 == 0 || shape.1 == 0 || (shape.0 != 1 && shape.1 != 1) {
                    return CalcResult::Error {
                        error: Error::VALUE,
                        origin: cell,
                        message: "Second argument must be a vector".to_string(),
                    };
                }
                LookupVector::Array(rows)
            }
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::Error {
                    error: Error::NA,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };
        // return array: a range or an array value with the shape of lookup_array
        let result = match self.evaluate_lookup_vector_node(&args[2], cell) {
            CalcResult::Range { left, right } => LookupVector::Range { left, right },
            CalcResult::Array(rows) => LookupVector::Array(rows),
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Range expected".to_string(),
                }
            }
        };
        if result.shape() != lookup.shape() {
            return CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Arrays must be of the same size".to_string(),
            };
        }
        let array = match self.lookup_vector_values(lookup, cell) {
            Ok(array) => array,
            Err(error) => return error,
        };
        let index = match search_mode {
            SearchMode::StartAtFirstItem | SearchMode::StartAtLastItem => {
                linear_search(&lookup_value, &array, search_mode, match_mode)
            }
            SearchMode::BinarySearchAscending | SearchMode::BinarySearchDescending => {
                let index = if match_mode == MatchMode::ExactMatchLarger {
                    if search_mode == SearchMode::BinarySearchAscending {
                        binary_search_or_greater(&lookup_value, &array)
                    } else {
                        binary_search_descending_or_greater(&lookup_value, &array)
                    }
                } else if search_mode == SearchMode::BinarySearchAscending {
                    binary_search_or_smaller(&lookup_value, &array)
                } else {
                    binary_search_descending_or_smaller(&lookup_value, &array)
                };
                match index {
                    Some(_) if match_mode == MatchMode::WildcardMatch => {
                        return CalcResult::Error {
                            error: Error::VALUE,
                            origin: cell,
                            message: "Cannot use wildcard in binary search".to_string(),
                        };
                    }
                    Some(l) => Some(l as usize).filter(|&l| {
                        match_mode != MatchMode::ExactMatch
                            || compare_values(&array[l], &lookup_value) == 0
                    }),
                    None => None,
                }
            }
        };
        match index {
            Some(index) => self.lookup_vector_item(result, index, cell),
            None => if_not_found,
        }
    }

    /// Reads all the values of a lookup vector, in order.
    /// A range of a whole column or a whole row stops at the used range of its sheet.
    // A one-cell reference is a range of one cell, as in Excel, not the value of the cell
    fn evaluate_lookup_vector_node(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        match node {
            Node::ReferenceKind { .. } => {
                let range = self.evaluate_node_with_reference(node, cell);
                if let CalcResult::Range { left, .. } = &range {
                    self.support
                        .entry(cell)
                        .or_default()
                        .push(CellOrRange::Cell((left.sheet, left.row, left.column)));
                }
                range
            }
            _ => self.evaluate_node_in_context(node, cell),
        }
    }

    fn lookup_vector_values(
        &mut self,
        vector: LookupVector,
        cell: CellReferenceIndex,
    ) -> Result<Vec<CalcResult>, CalcResult> {
        match vector {
            LookupVector::Range { left, right } => {
                let is_column = left.row != right.row;
                let mut row2 = right.row;
                let mut column2 = right.column;
                if left.row == 1 && row2 == LAST_ROW {
                    row2 = self.sheet_dimension(left.sheet, cell)?.max_row;
                }
                if left.column == 1 && column2 == LAST_COLUMN {
                    column2 = self.sheet_dimension(left.sheet, cell)?.max_column;
                }
                let right = CellReferenceIndex {
                    sheet: left.sheet,
                    column: column2,
                    row: row2,
                };
                Ok(self.prepare_array(&left, &right, is_column))
            }
            LookupVector::Array(rows) => Ok(rows
                .iter()
                .flatten()
                .map(|node| array_node_to_calc_result(node, cell))
                .collect()),
        }
    }

    /// Reads the item at `index` of a lookup vector.
    fn lookup_vector_item(
        &mut self,
        vector: LookupVector,
        index: usize,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match vector {
            LookupVector::Range { left, right } => {
                let is_column = left.row != right.row;
                let offset = index as i32;
                self.evaluate_cell(CellReferenceIndex {
                    sheet: left.sheet,
                    row: left.row + if is_column { offset } else { 0 },
                    column: left.column + if is_column { 0 } else { offset },
                })
            }
            LookupVector::Array(rows) => {
                // The shape check makes `index` valid for a single row or a single column.
                let node = if rows.len() == 1 {
                    &rows[0][index]
                } else {
                    &rows[index][0]
                };
                array_node_to_calc_result(node, cell)
            }
        }
    }

    fn sheet_dimension(
        &self,
        sheet: u32,
        cell: CellReferenceIndex,
    ) -> Result<WorksheetDimension, CalcResult> {
        match self.workbook.worksheet(sheet) {
            Ok(s) => Ok(s.dimension()),
            Err(_) => Err(CalcResult::new_error(
                Error::ERROR,
                cell,
                format!("Invalid worksheet index: '{sheet}'"),
            )),
        }
    }
}
