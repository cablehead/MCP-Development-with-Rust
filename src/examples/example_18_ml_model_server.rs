// Example 18: ML Model Server Implementation
//
// This example demonstrates how to build a machine learning model server
// with inference capabilities, model management, and prediction endpoints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

// Struct: ModelInput
//
// Represents input data for model inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInput {
    features: Vec<f64>,
    metadata: HashMap<String, String>,
}

// Struct: ModelOutput
//
// Represents model prediction output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOutput {
    prediction: f64,
    confidence: f64,
    model_version: String,
    inference_time_ms: u64,
}

// Struct: Model
//
// Represents a machine learning model.
#[derive(Debug, Clone)]
pub struct Model {
    id: Uuid,
    name: String,
    version: String,
    weights: Vec<f64>, // Simplified linear model weights
    bias: f64,
    is_active: bool,
}

impl Model {
    pub fn new(name: String, version: String, weights: Vec<f64>, bias: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            version,
            weights,
            bias,
            is_active: true,
        }
    }

    pub fn predict(&self, input: &ModelInput) -> ModelOutput {
        let start_time = std::time::Instant::now();

        // Simple linear model: y = w1*x1 + w2*x2 + ... + bias
        let mut prediction = self.bias;
        for (i, &feature) in input.features.iter().enumerate() {
            if i < self.weights.len() {
                prediction += self.weights[i] * feature;
            }
        }

        // Apply sigmoid for confidence score
        let confidence = 1.0 / (1.0 + (-prediction.abs()).exp());

        let inference_time = start_time.elapsed().as_millis() as u64;

        ModelOutput {
            prediction,
            confidence,
            model_version: self.version.clone(),
            inference_time_ms: inference_time,
        }
    }
}

// Struct: ModelServer
//
// Main ML model server that manages models and handles inference requests.
pub struct ModelServer {
    models: HashMap<String, Model>,
    active_model: Option<String>,
    inference_count: u64,
    total_inference_time: u64,
}

impl Default for ModelServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelServer {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            active_model: None,
            inference_count: 0,
            total_inference_time: 0,
        }
    }

    pub fn register_model(&mut self, model: Model) -> Result<(), String> {
        let model_key = format!("{}:{}", model.name, model.version);

        if self.models.contains_key(&model_key) {
            return Err("Model already exists".to_string());
        }

        info!("Registering model: {} ({})", model.name, model.version);
        self.models.insert(model_key.clone(), model);

        // Set as active if no active model
        if self.active_model.is_none() {
            self.active_model = Some(model_key);
        }

        Ok(())
    }

    pub fn set_active_model(&mut self, name: &str, version: &str) -> Result<(), String> {
        let model_key = format!("{}:{}", name, version);

        if !self.models.contains_key(&model_key) {
            return Err("Model not found".to_string());
        }

        self.active_model = Some(model_key);
        info!("Active model set to: {}:{}", name, version);
        Ok(())
    }

    pub fn predict(&mut self, input: ModelInput) -> Result<ModelOutput, String> {
        let active_key = self.active_model.as_ref().ok_or("No active model set")?;

        let model = self
            .models
            .get(active_key)
            .ok_or("Active model not found")?;

        if !model.is_active {
            return Err("Active model is disabled".to_string());
        }

        let output = model.predict(&input);

        // Update statistics
        self.inference_count += 1;
        self.total_inference_time += output.inference_time_ms;

        info!(
            "Prediction made: {:.3} (confidence: {:.3})",
            output.prediction, output.confidence
        );

        Ok(output)
    }

    pub fn batch_predict(&mut self, inputs: Vec<ModelInput>) -> Result<Vec<ModelOutput>, String> {
        let mut outputs = Vec::new();

        for input in inputs {
            match self.predict(input) {
                Ok(output) => outputs.push(output),
                Err(e) => return Err(format!("Batch prediction failed: {}", e)),
            }
        }

        info!("Batch prediction completed: {} predictions", outputs.len());
        Ok(outputs)
    }

    pub fn get_model_info(&self, name: &str, version: &str) -> Option<ModelInfo> {
        let model_key = format!("{}:{}", name, version);
        self.models.get(&model_key).map(|model| ModelInfo {
            id: model.id,
            name: model.name.clone(),
            version: model.version.clone(),
            is_active: model.is_active,
            is_current_active: self.active_model.as_ref() == Some(&model_key),
        })
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.models
            .iter()
            .map(|(key, model)| ModelInfo {
                id: model.id,
                name: model.name.clone(),
                version: model.version.clone(),
                is_active: model.is_active,
                is_current_active: self.active_model.as_ref() == Some(key),
            })
            .collect()
    }

    pub fn get_statistics(&self) -> ServerStatistics {
        let avg_inference_time = if self.inference_count > 0 {
            self.total_inference_time as f64 / self.inference_count as f64
        } else {
            0.0
        };

        ServerStatistics {
            total_models: self.models.len(),
            inference_count: self.inference_count,
            average_inference_time_ms: avg_inference_time,
            active_model: self.active_model.clone(),
        }
    }
}

// Struct: ModelInfo
//
// Contains metadata about a model.
#[derive(Debug, Serialize)]
pub struct ModelInfo {
    id: Uuid,
    name: String,
    version: String,
    is_active: bool,
    is_current_active: bool,
}

// Struct: ServerStatistics
//
// Contains server performance statistics.
#[derive(Debug, Serialize)]
pub struct ServerStatistics {
    total_models: usize,
    inference_count: u64,
    average_inference_time_ms: f64,
    active_model: Option<String>,
}

// Function: demo_ml_server
//
// Demonstrates the ML model server functionality.
fn demo_ml_server() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Creating ML Model Server ===");
    let mut server = ModelServer::new();

    // Register models
    let linear_model_v1 = Model::new(
        "linear_classifier".to_string(),
        "v1.0".to_string(),
        vec![0.5, -0.3, 0.8, 0.2],
        -0.1,
    );

    let linear_model_v2 = Model::new(
        "linear_classifier".to_string(),
        "v2.0".to_string(),
        vec![0.6, -0.2, 0.9, 0.1],
        0.05,
    );

    server.register_model(linear_model_v1)?;
    server.register_model(linear_model_v2)?;

    info!("=== Model Information ===");
    for model_info in server.list_models() {
        info!(
            "Model: {} {} (active: {}, current: {})",
            model_info.name, model_info.version, model_info.is_active, model_info.is_current_active
        );
    }

    info!("=== Single Predictions ===");

    // Make single predictions
    let input1 = ModelInput {
        features: vec![1.0, 2.0, -0.5, 0.8],
        metadata: HashMap::new(),
    };

    let output1 = server.predict(input1)?;
    info!(
        "Prediction 1: {:.3} (confidence: {:.3})",
        output1.prediction, output1.confidence
    );

    // Switch to v2 model
    server.set_active_model("linear_classifier", "v2.0")?;

    let input2 = ModelInput {
        features: vec![0.5, 1.5, -1.0, 0.3],
        metadata: HashMap::new(),
    };

    let output2 = server.predict(input2)?;
    info!(
        "Prediction 2 (v2.0): {:.3} (confidence: {:.3})",
        output2.prediction, output2.confidence
    );

    info!("=== Batch Predictions ===");

    let batch_inputs = vec![
        ModelInput {
            features: vec![1.2, 0.8, 0.5, -0.2],
            metadata: HashMap::new(),
        },
        ModelInput {
            features: vec![-0.5, 2.1, 0.3, 0.9],
            metadata: HashMap::new(),
        },
        ModelInput {
            features: vec![0.0, 1.0, -0.8, 0.4],
            metadata: HashMap::new(),
        },
    ];

    let batch_outputs = server.batch_predict(batch_inputs)?;
    for (i, output) in batch_outputs.iter().enumerate() {
        info!(
            "Batch prediction {}: {:.3} (confidence: {:.3})",
            i + 1,
            output.prediction,
            output.confidence
        );
    }

    let stats = server.get_statistics();
    info!("=== Server Statistics ===");
    info!("Total models: {}", stats.total_models);
    info!("Total inferences: {}", stats.inference_count);
    info!(
        "Average inference time: {:.2}ms",
        stats.average_inference_time_ms
    );
    info!("Active model: {:?}", stats.active_model);

    Ok(())
}

// Function: main
//
// Entry point demonstrating the ML model server implementation.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting ML Model Server Example");
    demo_ml_server()?;
    info!("ML Model Server Example completed successfully");

    Ok(())
}
