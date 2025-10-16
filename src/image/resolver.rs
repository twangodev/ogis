use hickory_resolver::{Resolver, TokioResolver};
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

/// DNS resolver that only allows global (public) IP addresses
#[derive(Clone)]
pub struct GlobalResolver {
    inner: Arc<TokioResolver>,
}

impl GlobalResolver {
    pub fn new() -> io::Result<Self> {
        let resolver: TokioResolver = Resolver::builder_tokio()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .build();

        Ok(Self {
            inner: Arc::new(resolver),
        })
    }
}

impl Resolve for GlobalResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.inner.clone();
        let name_str = name.as_str().to_string();

        Box::pin(async move {
            // Resolve DNS
            let lookup = resolver.lookup_ip(name.as_str()).await.map_err(|e| {
                Box::new(io::Error::new(io::ErrorKind::Other, e))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;

            let mut addrs = Vec::new();

            // Check all resolved IPs
            for ip in lookup.iter() {
                // Block private/local IPs
                if !ip_rfc::global(&ip) {
                    tracing::warn!(
                        "Blocked DNS resolution for {} to private IP: {}",
                        name_str,
                        ip
                    );
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "SSRF protection: hostname {} resolves to private IP {}",
                            name_str, ip
                        ),
                    ))
                        as Box<dyn std::error::Error + Send + Sync>);
                }

                // All IPs are public, add to socket addresses
                addrs.push(SocketAddr::new(ip, 0));
            }

            if addrs.is_empty() {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No IP addresses resolved for {}", name_str),
                ))
                    as Box<dyn std::error::Error + Send + Sync>);
            }

            tracing::debug!("Resolved {} to {} public IP(s)", name_str, addrs.len());

            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}
