use measure::MeasureRegistry;
use measure::Measures;
use std::time::SystemTime;

/// Errors returned by the exporter. Contain a string describing the cause of
/// the error.
#[derive(Debug)]
pub struct ExportError(pub String);

/// An interface to a data sink accepting accumulated expectation values.
pub trait Exporter {
    /// Performs a single export operation. Note that it does not reset the
    /// accumulated values, which is the job of the simulation engine.
    fn export(&mut self, measures: &Measures) -> Result<(), ExportError>;
}

/// Keeps a copy of measures. On `export(..)`, merges the reported data and
/// outputs the accumulated values to stdout.
pub struct DebugExporter {
    aggregated: MeasureRegistry,
    creation_timestamp: SystemTime,
}

impl DebugExporter {
    /// Constructs a new DebugExporter.
    pub fn new() -> DebugExporter {
        DebugExporter {
            aggregated: MeasureRegistry::new(),
            creation_timestamp: SystemTime::now(),
        }
    }

    /// Format the results in a pretty table.
    fn pretty_table(measures: &Measures) -> ::prettytable::Table {
        use prettytable::cell::Cell;
        use prettytable::format::Alignment;
        use prettytable::row::Row;
        use prettytable::Table;
        let mut table = Table::new();
        table.set_format(*::prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(Row::new(vec![
            Cell::new_align("MEASURE", Alignment::CENTER),
            Cell::new_align("EXPECTATION", Alignment::CENTER),
            Cell::new_align("UNCERTAINTY", Alignment::CENTER),
            Cell::new_align("RELATIVE UNCERTAINTY", Alignment::CENTER),
        ]));
        for measure in measures.slice() {
            let expectation = format!("{}", measure.acc.value());
            let uncertainty = format!("{}", measure.acc.uncertainty());
            let relative_uncertainty =
                format!("{}", measure.acc.uncertainty() / measure.acc.value().abs());
            table.add_row(Row::new(vec![
                Cell::new_align(&measure.name, Alignment::RIGHT),
                Cell::new(&expectation),
                Cell::new(&uncertainty),
                Cell::new(&relative_uncertainty),
            ]));
        }
        table
    }
}

impl Exporter for DebugExporter {
    fn export(&mut self, measures: &Measures) -> Result<(), ExportError> {
        let mut samples_processed: usize = 0;
        // Merge the reported values to the global accumulated values.
        for measure in measures.slice() {
            let measure_idx = match self.aggregated.find(&measure.name) {
                Some(idx) => idx,
                None => self.aggregated.register(measure.name.clone()),
            };
            self.aggregated
                .accumulator(measure_idx)
                .merge(measure.acc.clone());
            samples_processed = self.aggregated.accumulator(measure_idx).num_of_samples() as usize;
        }

        // Output the global accumulated values to stdout.
        println!();
        println!(
            "Simulation uptime: {} secs",
            self.creation_timestamp.elapsed().unwrap().as_secs()
        );
        println!("Samples processed: {}", samples_processed);
        println!("Aggregate values:");
        DebugExporter::pretty_table(self.aggregated.measures()).printstd();
        Ok(())
    }
}
