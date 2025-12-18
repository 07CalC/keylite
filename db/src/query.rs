use serde_json::Value;

use crate::{db::KeyLite, error::Result, filter::Filter, get_field};

pub struct Sort {
    field: String,
    ascending: bool,
}

pub struct Query<'a> {
    db: &'a KeyLite,
    collection: String,
    filters: Vec<Filter>,
    sort: Option<Sort>,
    limit: Option<usize>,
    skip: Option<usize>,
}

impl<'a> Query<'a> {
    pub fn new(db: &'a KeyLite, collection: &str) -> Self {
        Self {
            db,
            collection: collection.to_string(),
            filters: Vec::new(),
            limit: None,
            skip: None,
            sort: None,
        }
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn sort(mut self, field: &str, ascending: bool) -> Self {
        self.sort = Some(Sort {
            field: field.to_string(),
            ascending,
        });
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    // this should be optmised using indexes
    pub fn execute(&self) -> Result<Vec<Value>> {
        let mut results = self.db.scan_collection(&self.collection)?;

        if !self.filters.is_empty() {
            results.retain(|doc| self.filters.iter().all(|filter| filter.matches(doc)));
        }

        if let Some(ref sort) = self.sort {
            results.sort_by(|a, b| {
                let a_val = get_field(a, &sort.field);
                let b_val = get_field(b, &sort.field);

                let cmp = match (a_val, b_val) {
                    (Some(Value::Number(a_num)), Some(Value::Number(b_num))) => {
                        match (a_num.as_f64(), b_num.as_f64()) {
                            (Some(a_f), Some(b_f)) => {
                                a_f.partial_cmp(&b_f).unwrap_or(std::cmp::Ordering::Equal)
                            }
                            _ => std::cmp::Ordering::Equal,
                        }
                    }
                    (Some(Value::String(a_str)), Some(Value::String(b_str))) => a_str.cmp(b_str),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    _ => std::cmp::Ordering::Equal,
                };

                if sort.ascending { cmp } else { cmp.reverse() }
            });
        }

        let skip = self.skip.unwrap_or(0);
        if skip > 0 {
            results = results.into_iter().skip(skip).collect();
        }

        if let Some(limit) = self.limit {
            results.truncate(limit);
        }

        Ok(results)
    }
}
