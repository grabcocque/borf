use arrow2::array::{MutablePrimitiveArray, MutableUtf8Array};
use arrow2::datatypes::{DataType, Schema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single scalar value for a memtable row.
#[derive(Debug, Serialize, Deserialize)]
pub enum ScalarValue {
    Int64(i64),
    Float64(f64),
    Utf8(String),
}

/// In-memory columnar buffer using Arrow2 builders.
/// Internal builder types per column.
#[derive(Clone)]
pub enum ColumnBuilder {
    Int64(MutablePrimitiveArray<i64>),
    Float64(MutablePrimitiveArray<f64>),
    Utf8(MutableUtf8Array<i32>),
}

/// In-memory columnar buffer using Arrow2 builders.
pub struct Memtable {
    schema: Schema,
    builders: HashMap<String, ColumnBuilder>,
    row_count: usize,
}

impl Memtable {
    /// Create a new Memtable with the given schema and capacity (rows).
    pub fn new(schema: Schema, capacity: usize) -> Self {
        let mut builders: HashMap<String, ColumnBuilder> = HashMap::new();
        for field in &schema.fields {
            let builder = match &field.data_type {
                DataType::Int64 => {
                    ColumnBuilder::Int64(MutablePrimitiveArray::<i64>::with_capacity(capacity))
                }
                DataType::Float64 => {
                    ColumnBuilder::Float64(MutablePrimitiveArray::<f64>::with_capacity(capacity))
                }
                DataType::Utf8 => {
                    ColumnBuilder::Utf8(MutableUtf8Array::<i32>::with_capacity(capacity))
                }
                dt => panic!("Unsupported data type: {:?}", dt),
            };
            builders.insert(field.name.clone(), builder);
        }
        Self {
            schema,
            builders,
            row_count: 0,
        }
    }

    /// Append a single row of values to the memtable.
    pub fn append_row(&mut self, row: Vec<ScalarValue>) {
        assert_eq!(
            row.len(),
            self.schema.fields.len(),
            "Row length must match schema"
        );
        for (field, value) in self.schema.fields.iter().zip(row.into_iter()) {
            let builder = self.builders.get_mut(&field.name).unwrap();
            match (builder, value) {
                (ColumnBuilder::Int64(b), ScalarValue::Int64(v)) => b.push(Some(v)),
                (ColumnBuilder::Float64(b), ScalarValue::Float64(v)) => b.push(Some(v)),
                (ColumnBuilder::Utf8(b), ScalarValue::Utf8(v)) => b.push(Some(&v)),
                _ => panic!("Type mismatch for field {}", field.name),
            }
        }
        self.row_count += 1;
    }

    /// Number of rows currently buffered.
    pub fn row_count(&self) -> usize {
        self.row_count
    }
    /// Extract all columns as boxed arrays, consuming builders.
    pub fn to_arrays(&mut self) -> (Vec<Box<dyn arrow2::array::Array>>, usize) {
        use arrow2::array::MutableArray;
        let row_count = self.row_count;
        let mut arrays = Vec::with_capacity(self.builders.len());
        for field in &self.schema.fields {
            let builder = self.builders.get_mut(&field.name).unwrap();
            let array: Box<dyn arrow2::array::Array> = match builder {
                ColumnBuilder::Int64(ref mut b) => b.as_box(),
                ColumnBuilder::Float64(ref mut b) => b.as_box(),
                ColumnBuilder::Utf8(ref mut b) => b.as_box(),
            };
            arrays.push(array);
        }
        (arrays, row_count)
    }
}
// Unit tests for Memtable
#[cfg(test)]
mod tests {
    use super::*;
    use arrow2::datatypes::{DataType, Field, Schema};

    #[test]
    fn test_memtable_basic_append() {
        // Define schema: int, float, utf8 columns
        // Build schema manually (no Schema::new in arrow2):
        let fields = vec![
            Field::new("i", DataType::Int64, false),
            Field::new("f", DataType::Float64, false),
            Field::new("s", DataType::Utf8, false),
        ];
        let schema = Schema {
            fields: fields.clone(),
            metadata: Default::default(),
        };
        let mut mt = Memtable::new(schema.clone(), 2);
        assert_eq!(mt.row_count(), 0);

        // Append first row
        mt.append_row(vec![
            ScalarValue::Int64(42),
            // Use precise constant for PI
            ScalarValue::Float64(std::f64::consts::PI),
            ScalarValue::Utf8("foo".to_string()),
        ]);
        assert_eq!(mt.row_count(), 1);

        // Append second row
        mt.append_row(vec![
            ScalarValue::Int64(-1),
            // Use precise constant for Euler's number
            ScalarValue::Float64(std::f64::consts::E),
            ScalarValue::Utf8("bar".to_string()),
        ]);
        assert_eq!(mt.row_count(), 2);

        // Verify builder lengths only
        use arrow2::array::MutableArray;
        // Int64
        if let ColumnBuilder::Int64(ref b) = mt.builders.get("i").unwrap() {
            assert_eq!(b.len(), 2);
        } else {
            panic!("Expected Int64 builder");
        }
        // Float64
        if let ColumnBuilder::Float64(ref b) = mt.builders.get("f").unwrap() {
            assert_eq!(b.len(), 2);
        } else {
            panic!("Expected Float64 builder");
        }
        // Utf8
        if let ColumnBuilder::Utf8(ref b) = mt.builders.get("s").unwrap() {
            assert_eq!(b.len(), 2);
        } else {
            panic!("Expected Utf8 builder");
        }
    }
}
