use crate::common::constant::DEFAULT_XXL_NAMESPACE;
use std::sync::Arc;

pub struct ParsedRegistryKey {
    pub namespace: Arc<String>,
    pub app_name: Arc<String>,
}

pub fn parse_registry_key(registry_key: &str) -> ParsedRegistryKey {
    const NS_PREFIX: &str = "ns://";

    if let Some(rest) = registry_key.strip_prefix(NS_PREFIX) {
        if let Some(separator_pos) = rest.find('/') {
            let namespace = &rest[..separator_pos];
            let app_name = &rest[separator_pos + 1..];
            if !namespace.is_empty() && !app_name.is_empty() {
                return ParsedRegistryKey {
                    namespace: Arc::new(namespace.to_string()),
                    app_name: Arc::new(app_name.to_string()),
                };
            }
        }
    }

    if let Some(separator_pos) = registry_key.find("@@") {
        let namespace = &registry_key[..separator_pos];
        let app_name = &registry_key[separator_pos + 2..];
        if !namespace.is_empty() && !app_name.is_empty() {
            return ParsedRegistryKey {
                namespace: Arc::new(namespace.to_string()),
                app_name: Arc::new(app_name.to_string()),
            };
        }
    }

    ParsedRegistryKey {
        namespace: DEFAULT_XXL_NAMESPACE.clone(),
        app_name: Arc::new(registry_key.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_namespace() {
        let result = parse_registry_key("ns://my-ns/my-app");
        assert_eq!(result.namespace.as_str(), "my-ns");
        assert_eq!(result.app_name.as_str(), "my-app");
    }

    #[test]
    fn test_parse_with_namespace_style2() {
        let result = parse_registry_key("my-ns@@my-app");
        assert_eq!(result.namespace.as_str(), "my-ns");
        assert_eq!(result.app_name.as_str(), "my-app");
    }

    #[test]
    fn test_parse_without_namespace() {
        let result = parse_registry_key("my-app");
        assert_eq!(result.namespace.as_str(), DEFAULT_XXL_NAMESPACE.as_str());
        assert_eq!(result.app_name.as_str(), "my-app");
    }

    #[test]
    fn test_parse_empty_string() {
        let result = parse_registry_key("");
        assert_eq!(result.namespace.as_str(), DEFAULT_XXL_NAMESPACE.as_str());
        assert_eq!(result.app_name.as_str(), "");
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = parse_registry_key("ns://");
        assert_eq!(result.namespace.as_str(), DEFAULT_XXL_NAMESPACE.as_str());
        assert_eq!(result.app_name.as_str(), "ns://");
    }

    #[test]
    fn test_parse_missing_slash() {
        let result = parse_registry_key("ns://my-ns");
        assert_eq!(result.namespace.as_str(), DEFAULT_XXL_NAMESPACE.as_str());
        assert_eq!(result.app_name.as_str(), "ns://my-ns");
    }

    #[test]
    fn test_parse_empty_namespace() {
        let result = parse_registry_key("ns:///my-app");
        assert_eq!(result.namespace.as_str(), DEFAULT_XXL_NAMESPACE.as_str());
        assert_eq!(result.app_name.as_str(), "ns:///my-app");
    }

    #[test]
    fn test_parse_empty_app_name() {
        let result = parse_registry_key("ns://my-ns/");
        assert_eq!(result.namespace.as_str(), DEFAULT_XXL_NAMESPACE.as_str());
        assert_eq!(result.app_name.as_str(), "ns://my-ns/");
    }
}
