// Example 15: Data Pipeline Implementation
//
// This example demonstrates how to build a data pipeline for processing,
// transforming, and loading data from various sources.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

// Struct: DataRecord
//
// Represents a single data record flowing through the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    id: Uuid,
    timestamp: DateTime<Utc>,
    source: String,
    data: HashMap<String, serde_json::Value>,
}

impl DataRecord {
    pub fn new(source: String, data: HashMap<String, serde_json::Value>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            source,
            data,
        }
    }
}

// Enum: TransformOperation
//
// Defines different types of transformations that can be applied to data.
#[derive(Debug, Clone)]
pub enum TransformOperation {
    Filter {
        field: String,
        min_value: f64,
    },
    Map {
        input_field: String,
        output_field: String,
        multiplier: f64,
    },
    Enrich {
        field: String,
        value: serde_json::Value,
    },
}

impl TransformOperation {
    pub fn apply(&self, mut record: DataRecord) -> Result<DataRecord, String> {
        match self {
            TransformOperation::Filter { field, min_value } => {
                if let Some(value) = record.data.get(field) {
                    if let Some(num) = value.as_f64() {
                        if num >= *min_value {
                            Ok(record)
                        } else {
                            Err("Value below threshold".to_string())
                        }
                    } else {
                        Err("Field is not a number".to_string())
                    }
                } else {
                    Err("Field not found".to_string())
                }
            }
            TransformOperation::Map {
                input_field,
                output_field,
                multiplier,
            } => {
                if let Some(value) = record.data.get(input_field) {
                    if let Some(num) = value.as_f64() {
                        let result = num * multiplier;
                        record.data.insert(
                            output_field.clone(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(result).unwrap(),
                            ),
                        );
                        Ok(record)
                    } else {
                        Err("Input field is not a number".to_string())
                    }
                } else {
                    Err("Input field not found".to_string())
                }
            }
            TransformOperation::Enrich { field, value } => {
                record.data.insert(field.clone(), value.clone());
                Ok(record)
            }
        }
    }
}

// Struct: DataPipeline
//
// Main pipeline that processes data through multiple transformation stages.
pub struct DataPipeline {
    transformations: Vec<TransformOperation>,
    processed_count: u64,
    error_count: u64,
}

impl Default for DataPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl DataPipeline {
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
            processed_count: 0,
            error_count: 0,
        }
    }

    pub fn add_transformation(&mut self, transform: TransformOperation) {
        self.transformations.push(transform);
    }

    pub fn process_record(&mut self, mut record: DataRecord) -> Result<DataRecord, String> {
        for transform in &self.transformations {
            match transform.apply(record) {
                Ok(transformed) => record = transformed,
                Err(e) => {
                    self.error_count += 1;
                    return Err(e);
                }
            }
        }
        self.processed_count += 1;
        Ok(record)
    }

    pub fn get_statistics(&self) -> (u64, u64) {
        (self.processed_count, self.error_count)
    }
}

// Function: create_sample_data
//
// Creates sample data records for testing the pipeline.
fn create_sample_data() -> Vec<DataRecord> {
    vec![
        {
            let mut data = HashMap::new();
            data.insert(
                "temperature".to_string(),
                serde_json::Value::Number(serde_json::Number::from(25)),
            );
            data.insert(
                "humidity".to_string(),
                serde_json::Value::Number(serde_json::Number::from(60)),
            );
            DataRecord::new("weather_station".to_string(), data)
        },
        {
            let mut data = HashMap::new();
            data.insert(
                "temperature".to_string(),
                serde_json::Value::Number(serde_json::Number::from(18)),
            );
            data.insert(
                "humidity".to_string(),
                serde_json::Value::Number(serde_json::Number::from(75)),
            );
            DataRecord::new("weather_station".to_string(), data)
        },
        {
            let mut data = HashMap::new();
            data.insert(
                "temperature".to_string(),
                serde_json::Value::Number(serde_json::Number::from(32)),
            );
            data.insert(
                "humidity".to_string(),
                serde_json::Value::Number(serde_json::Number::from(45)),
            );
            DataRecord::new("weather_station".to_string(), data)
        },
    ]
}

// Function: demo_data_pipeline
//
// Demonstrates the data pipeline functionality.
fn demo_data_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Creating Data Pipeline ===");

    let mut pipeline = DataPipeline::new();

    // Add transformations
    pipeline.add_transformation(TransformOperation::Filter {
        field: "temperature".to_string(),
        min_value: 20.0,
    });

    pipeline.add_transformation(TransformOperation::Map {
        input_field: "temperature".to_string(),
        output_field: "temperature_fahrenheit".to_string(),
        multiplier: 9.0 / 5.0,
    });

    pipeline.add_transformation(TransformOperation::Enrich {
        field: "processed_at".to_string(),
        value: serde_json::Value::String(Utc::now().to_rfc3339()),
    });

    info!("=== Processing Sample Data ===");

    let sample_data = create_sample_data();
    let mut results = Vec::new();

    for record in sample_data {
        info!(
            "Processing record from {}: {:?}",
            record.source, record.data
        );

        match pipeline.process_record(record) {
            Ok(processed_record) => {
                info!("✅ Processed successfully: {:?}", processed_record.data);
                results.push(processed_record);
            }
            Err(e) => {
                warn!("❌ Processing failed: {}", e);
            }
        }
    }

    let (processed_count, error_count) = pipeline.get_statistics();
    info!("=== Pipeline Statistics ===");
    info!("Processed: {}", processed_count);
    info!("Errors: {}", error_count);

    Ok(())
}

// Function: main
//
// Entry point demonstrating the data pipeline implementation.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Data Pipeline Example");
    demo_data_pipeline()?;
    info!("Data Pipeline Example completed successfully");

    Ok(())
}
