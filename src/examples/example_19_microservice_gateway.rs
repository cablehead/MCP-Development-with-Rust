// Example 19: Microservice Gateway Implementation
//
// This example demonstrates how to build a microservice gateway with
// service routing, load balancing, and basic service discovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{info, warn};
use uuid::Uuid;

// Struct: ServiceEndpoint
//
// Represents a service endpoint in the gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    id: Uuid,
    service_name: String,
    host: String,
    port: u16,
    health_check_url: String,
    is_healthy: bool,
    weight: u32,
}

impl ServiceEndpoint {
    pub fn new(service_name: String, host: String, port: u16) -> Self {
        Self {
            id: Uuid::new_v4(),
            service_name: service_name.clone(),
            host: host.clone(),
            port,
            health_check_url: format!("http://{}:{}/health", host, port),
            is_healthy: true,
            weight: 1,
        }
    }
}

// Struct: GatewayRequest
//
// Represents an incoming request to the gateway.
#[derive(Debug, Clone)]
pub struct GatewayRequest {
    id: Uuid,
    service_name: String,
    path: String,
    #[allow(dead_code)]
    method: String,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
    #[allow(dead_code)]
    body: Option<String>,
}

impl GatewayRequest {
    pub fn new(service_name: String, path: String, method: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            service_name,
            path,
            method,
            headers: HashMap::new(),
            body: None,
        }
    }
}

// Struct: GatewayResponse
//
// Represents a response from the gateway.
#[derive(Debug, Clone)]
pub struct GatewayResponse {
    #[allow(dead_code)]
    request_id: Uuid,
    #[allow(dead_code)]
    status_code: u16,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
    #[allow(dead_code)]
    body: String,
    response_time_ms: u64,
    service_endpoint: String,
}

// Enum: LoadBalancingStrategy
//
// Defines different load balancing strategies.
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    Random,
}

// Struct: ServiceRegistry
//
// Manages service discovery and health checking.
pub struct ServiceRegistry {
    services: HashMap<String, Vec<ServiceEndpoint>>,
    round_robin_counters: HashMap<String, AtomicUsize>,
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            round_robin_counters: HashMap::new(),
        }
    }

    pub fn register_service(&mut self, endpoint: ServiceEndpoint) {
        let service_name = endpoint.service_name.clone();

        self.services
            .entry(service_name.clone())
            .or_default()
            .push(endpoint);

        self.round_robin_counters
            .entry(service_name.clone())
            .or_insert_with(|| AtomicUsize::new(0));

        info!(
            "Registered service endpoint: {} at {}:{}",
            service_name,
            self.services[&service_name].last().unwrap().host,
            self.services[&service_name].last().unwrap().port
        );
    }

    pub fn get_healthy_endpoints(&self, service_name: &str) -> Vec<&ServiceEndpoint> {
        self.services
            .get(service_name)
            .map(|endpoints| {
                endpoints
                    .iter()
                    .filter(|endpoint| endpoint.is_healthy)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn select_endpoint(
        &self,
        service_name: &str,
        strategy: &LoadBalancingStrategy,
    ) -> Option<&ServiceEndpoint> {
        let healthy_endpoints = self.get_healthy_endpoints(service_name);

        if healthy_endpoints.is_empty() {
            return None;
        }

        match strategy {
            LoadBalancingStrategy::RoundRobin => {
                let counter = self.round_robin_counters.get(service_name)?;
                let index = counter.fetch_add(1, Ordering::Relaxed) % healthy_endpoints.len();
                healthy_endpoints.get(index).copied()
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                // Simplified: just use the first endpoint for demo
                healthy_endpoints.first().copied()
            }
            LoadBalancingStrategy::Random => {
                let index = rand::random::<usize>() % healthy_endpoints.len();
                healthy_endpoints.get(index).copied()
            }
        }
    }

    pub fn mark_endpoint_unhealthy(&mut self, endpoint_id: Uuid) {
        for endpoints in self.services.values_mut() {
            for endpoint in endpoints.iter_mut() {
                if endpoint.id == endpoint_id {
                    endpoint.is_healthy = false;
                    warn!("Marked endpoint {} as unhealthy", endpoint_id);
                    return;
                }
            }
        }
    }

    pub fn get_service_statistics(&self) -> HashMap<String, ServiceStats> {
        self.services
            .iter()
            .map(|(name, endpoints)| {
                let healthy_count = endpoints.iter().filter(|e| e.is_healthy).count();
                let total_count = endpoints.len();

                (
                    name.clone(),
                    ServiceStats {
                        total_endpoints: total_count,
                        healthy_endpoints: healthy_count,
                        unhealthy_endpoints: total_count - healthy_count,
                    },
                )
            })
            .collect()
    }
}

// Struct: ServiceStats
//
// Contains statistics about a service.
#[derive(Debug, Clone, Serialize)]
pub struct ServiceStats {
    total_endpoints: usize,
    healthy_endpoints: usize,
    unhealthy_endpoints: usize,
}

// Struct: MicroserviceGateway
//
// Main gateway that handles routing and load balancing.
pub struct MicroserviceGateway {
    service_registry: ServiceRegistry,
    load_balancing_strategy: LoadBalancingStrategy,
    request_count: u64,
    total_response_time: u64,
    route_mappings: HashMap<String, String>, // path prefix -> service name
}

impl MicroserviceGateway {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            service_registry: ServiceRegistry::new(),
            load_balancing_strategy: strategy,
            request_count: 0,
            total_response_time: 0,
            route_mappings: HashMap::new(),
        }
    }

    pub fn register_service(&mut self, endpoint: ServiceEndpoint) {
        self.service_registry.register_service(endpoint);
    }

    pub fn add_route(&mut self, path_prefix: String, service_name: String) {
        self.route_mappings
            .insert(path_prefix.clone(), service_name.clone());
        info!("Added route: {} -> {}", path_prefix, service_name);
    }

    pub fn resolve_service(&self, path: &str) -> Option<String> {
        // Find the longest matching prefix
        self.route_mappings
            .iter()
            .filter(|(prefix, _)| path.starts_with(*prefix))
            .max_by_key(|(prefix, _)| prefix.len())
            .map(|(_, service)| service.clone())
    }

    pub fn handle_request(
        &mut self,
        mut request: GatewayRequest,
    ) -> Result<GatewayResponse, String> {
        let start_time = std::time::Instant::now();

        // Resolve service from path if not explicitly set
        if request.service_name.is_empty() {
            request.service_name = self
                .resolve_service(&request.path)
                .ok_or("No route found for path")?;
        }

        // Select an endpoint using load balancing
        let endpoint = self
            .service_registry
            .select_endpoint(&request.service_name, &self.load_balancing_strategy)
            .ok_or("No healthy endpoints available")?;

        // Simulate request forwarding
        let response = self.forward_request(&request, endpoint)?;

        let response_time = start_time.elapsed().as_millis() as u64;

        // Update statistics
        self.request_count += 1;
        self.total_response_time += response_time;

        info!(
            "Request {} routed to {}:{} in {}ms",
            request.id, endpoint.host, endpoint.port, response_time
        );

        Ok(GatewayResponse {
            request_id: request.id,
            status_code: response.status_code,
            headers: response.headers,
            body: response.body,
            response_time_ms: response_time,
            service_endpoint: format!("{}:{}", endpoint.host, endpoint.port),
        })
    }

    fn forward_request(
        &self,
        request: &GatewayRequest,
        endpoint: &ServiceEndpoint,
    ) -> Result<MockResponse, String> {
        // Simulate request forwarding (in real implementation, use HTTP client)
        // Note: In async context, this would need to be awaited
        // tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; // Simulate network delay

        // Mock successful response
        Ok(MockResponse {
            status_code: 200,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers.insert("X-Service".to_string(), endpoint.service_name.clone());
                headers
            },
            body: format!(
                r#"{{"message": "Response from {}", "request_id": "{}"}}"#,
                endpoint.service_name, request.id
            ),
        })
    }

    pub fn get_statistics(&self) -> GatewayStatistics {
        let avg_response_time = if self.request_count > 0 {
            self.total_response_time as f64 / self.request_count as f64
        } else {
            0.0
        };

        GatewayStatistics {
            total_requests: self.request_count,
            average_response_time_ms: avg_response_time,
            service_stats: self.service_registry.get_service_statistics(),
            active_routes: self.route_mappings.len(),
        }
    }
}

// Struct: MockResponse
//
// Mock response for demonstration purposes.
struct MockResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

// Struct: GatewayStatistics
//
// Contains gateway performance statistics.
#[derive(Debug, Serialize)]
pub struct GatewayStatistics {
    total_requests: u64,
    average_response_time_ms: f64,
    service_stats: HashMap<String, ServiceStats>,
    active_routes: usize,
}

// Function: demo_microservice_gateway
//
// Demonstrates the microservice gateway functionality.
fn demo_microservice_gateway() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Creating Microservice Gateway ===");
    let mut gateway = MicroserviceGateway::new(LoadBalancingStrategy::RoundRobin);

    // Register services
    gateway.register_service(ServiceEndpoint::new(
        "user-service".to_string(),
        "localhost".to_string(),
        8001,
    ));

    gateway.register_service(ServiceEndpoint::new(
        "user-service".to_string(),
        "localhost".to_string(),
        8002,
    ));

    gateway.register_service(ServiceEndpoint::new(
        "order-service".to_string(),
        "localhost".to_string(),
        8003,
    ));

    // Add routes
    gateway.add_route("/api/users".to_string(), "user-service".to_string());
    gateway.add_route("/api/orders".to_string(), "order-service".to_string());

    info!("=== Processing Requests ===");

    // Handle multiple requests
    let requests = vec![
        GatewayRequest::new(
            "".to_string(),
            "/api/users/123".to_string(),
            "GET".to_string(),
        ),
        GatewayRequest::new(
            "".to_string(),
            "/api/users/456".to_string(),
            "GET".to_string(),
        ),
        GatewayRequest::new(
            "".to_string(),
            "/api/orders/789".to_string(),
            "POST".to_string(),
        ),
        GatewayRequest::new(
            "".to_string(),
            "/api/users/999".to_string(),
            "PUT".to_string(),
        ),
        GatewayRequest::new(
            "".to_string(),
            "/api/orders/111".to_string(),
            "GET".to_string(),
        ),
    ];

    for request in requests {
        match gateway.handle_request(request.clone()) {
            Ok(response) => {
                info!(
                    "✅ Request {} -> {} ({}ms)",
                    request.path, response.service_endpoint, response.response_time_ms
                );
            }
            Err(e) => {
                warn!("❌ Request {} failed: {}", request.path, e);
            }
        }
    }

    let stats = gateway.get_statistics();
    info!("=== Gateway Statistics ===");
    info!("Total requests: {}", stats.total_requests);
    info!(
        "Average response time: {:.2}ms",
        stats.average_response_time_ms
    );
    info!("Active routes: {}", stats.active_routes);

    info!("Service Statistics:");
    for (service, stats) in stats.service_stats {
        info!(
            "  {}: {}/{} healthy endpoints",
            service, stats.healthy_endpoints, stats.total_endpoints
        );
    }

    Ok(())
}

// Function: main
//
// Entry point demonstrating the microservice gateway implementation.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Microservice Gateway Example");
    demo_microservice_gateway()?;
    info!("Microservice Gateway Example completed successfully");

    Ok(())
}
