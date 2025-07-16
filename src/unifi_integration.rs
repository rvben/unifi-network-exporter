use serde::Deserialize;

// Integration API response wrapper
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct IntegrationResponse<T> {
    pub offset: u32,
    pub limit: u32,
    pub count: u32,
    #[serde(rename = "totalCount")]
    pub total_count: u32,
    pub data: Vec<T>,
}

// Site structure for Integration API
#[derive(Debug, Deserialize, Clone)]
pub struct IntegrationSite {
    pub id: String,
    #[serde(rename = "internalReference")]
    pub internal_reference: String,
    pub name: String,
}

// Note: IntegrationDevice and IntegrationClient structs were removed
// as we now use the regular API endpoints instead of Integration API

impl IntegrationSite {
    pub fn to_site(&self) -> crate::unifi::Site {
        crate::unifi::Site {
            _id: self.id.clone(),
            name: self.internal_reference.clone(),
            desc: self.name.clone(),
            attr_hidden_id: None,
            attr_no_delete: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_response_deserialize() {
        let json = r#"{
            "offset": 0,
            "limit": 50,
            "count": 2,
            "totalCount": 2,
            "data": []
        }"#;
        let response: IntegrationResponse<IntegrationSite> = serde_json::from_str(json).unwrap();
        assert_eq!(response.offset, 0);
        assert_eq!(response.limit, 50);
        assert_eq!(response.count, 2);
        assert_eq!(response.total_count, 2);
        assert_eq!(response.data.len(), 0);
    }

    #[test]
    fn test_integration_site_deserialize() {
        let json = r#"{
            "id": "88f7af54-98f8-306a-a1c7-c9349722b1f6",
            "internalReference": "default",
            "name": "Default Site"
        }"#;
        let site: IntegrationSite = serde_json::from_str(json).unwrap();
        assert_eq!(site.id, "88f7af54-98f8-306a-a1c7-c9349722b1f6");
        assert_eq!(site.internal_reference, "default");
        assert_eq!(site.name, "Default Site");
    }

    #[test]
    fn test_integration_site_to_site() {
        let int_site = IntegrationSite {
            id: "88f7af54-98f8-306a-a1c7-c9349722b1f6".to_string(),
            internal_reference: "default".to_string(),
            name: "Default Site".to_string(),
        };

        let site = int_site.to_site();
        assert_eq!(site._id, "88f7af54-98f8-306a-a1c7-c9349722b1f6");
        assert_eq!(site.name, "default");
        assert_eq!(site.desc, "Default Site");
        assert_eq!(site.attr_hidden_id, None);
        assert_eq!(site.attr_no_delete, None);
    }
}
