use crate::{
    value_iterator::{ExternalIterator2, ValueIterator, ValueIteratorOutput as Output},
    Value, Vm,
};

/// An iterator that links the output of two iterators together in a chained sequence
pub struct Chain {
    iter_a: Option<ValueIterator>,
    iter_b: ValueIterator,
}

impl Chain {
    pub fn new(iter_a: ValueIterator, iter_b: ValueIterator) -> Self {
        Self {
            iter_a: Some(iter_a),
            iter_b,
        }
    }
}

impl ExternalIterator2 for Chain {
    fn make_copy(&self) -> ValueIterator {
        let result = Self {
            iter_a: self.iter_a.as_ref().map(|iter| iter.make_copy()),
            iter_b: self.iter_b.make_copy(),
        };
        ValueIterator::make_external_2(result)
    }
}

impl Iterator for Chain {
    type Item = Output;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter_a {
            Some(ref mut iter) => match iter.next() {
                output @ Some(_) => output,
                None => {
                    self.iter_a = None;
                    self.iter_b.next()
                }
            },
            None => self.iter_b.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.iter_a {
            Some(iter_a) => {
                let (lower_a, upper_a) = iter_a.size_hint();
                let (lower_b, upper_b) = self.iter_b.size_hint();

                let lower = lower_a.saturating_add(lower_b);
                let upper = match (upper_a, upper_b) {
                    (Some(a), Some(b)) => a.checked_add(b),
                    _ => None,
                };

                (lower, upper)
            }
            None => self.iter_b.size_hint(),
        }
    }
}

/// An iterator that runs a function on each output value from the adapted iterator
pub struct Each {
    iter: ValueIterator,
    function: Value,
    vm: Vm,
}

impl Each {
    pub fn new(iter: ValueIterator, function: Value, vm: Vm) -> Self {
        Self { iter, function, vm }
    }
}

impl ExternalIterator2 for Each {
    fn make_copy(&self) -> ValueIterator {
        let result = Self {
            iter: self.iter.make_copy(),
            function: self.function.clone(),
            vm: self.vm.spawn_shared_vm(),
        };
        ValueIterator::make_external_2(result)
    }
}

impl Iterator for Each {
    type Item = Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(collect_pair) // TODO Does collecting the pair for iterator.each still make sense?
            .map(|output| match output {
                Output::Value(value) => match self.vm.run_function(self.function.clone(), &[value])
                {
                    Ok(result) => Output::Value(result),
                    Err(error) => Output::Error(error.with_prefix("iterator.each")),
                },
                other => other,
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

fn collect_pair(iterator_output: Output) -> Output {
    match iterator_output {
        Output::ValuePair(first, second) => Output::Value(Value::Tuple(vec![first, second].into())),
        _ => iterator_output,
    }
}

// See runtime/tests/iterator_adaptor_tests.rs for tests
