#[cfg(test)]
mod tests {
    use crate::scan_logger::ScanLogger;

    #[test]
    fn test_parse_user_agent_chrome() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let (browser, device) = ScanLogger::parse_user_agent(ua);

        assert_eq!(browser, Some("Chrome".to_string()));
        assert_eq!(device, Some("Desktop".to_string()));
    }

    #[test]
    fn test_parse_user_agent_safari_mobile() {
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Mobile/15E148 Safari/604.1";
        let (browser, device) = ScanLogger::parse_user_agent(ua);

        assert_eq!(browser, Some("Safari".to_string()));
        assert_eq!(device, Some("Mobile".to_string()));
    }

    #[test]
    fn test_parse_user_agent_firefox() {
        let ua = "Mozilla/5.0 (X11; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0";
        let (browser, device) = ScanLogger::parse_user_agent(ua);

        assert_eq!(browser, Some("Firefox".to_string()));
        assert_eq!(device, Some("Desktop".to_string()));
    }

    #[test]
    fn test_parse_user_agent_edge() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59";
        let (browser, device) = ScanLogger::parse_user_agent(ua);

        assert_eq!(browser, Some("Edge".to_string()));
        assert_eq!(device, Some("Desktop".to_string()));
    }

    #[test]
    fn test_parse_user_agent_tablet() {
        let ua = "Mozilla/5.0 (iPad; CPU OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Mobile/15E148 Safari/604.1";
        let (browser, device) = ScanLogger::parse_user_agent(ua);

        assert_eq!(device, Some("Tablet".to_string()));
    }

    #[test]
    fn test_parse_user_agent_unknown() {
        let ua = "Unknown Browser";
        let (browser, device) = ScanLogger::parse_user_agent(ua);

        assert_eq!(browser, None);
        assert_eq!(device, None);
    }
}
