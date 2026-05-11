use crate::*;

pub(crate) fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub(crate) fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

pub(crate) fn is_github_url(value: &str) -> bool {
    Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(|h| h.eq_ignore_ascii_case("github.com")))
        .unwrap_or(false)
}

pub(crate) fn url_domain(value: &str) -> Option<String> {
    Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_ascii_lowercase()))
}

pub(crate) fn classify_privacy(value: &str) -> PrivacyClass {
    let Ok(url) = Url::parse(value) else {
        return PrivacyClass::Ambiguous;
    };
    if url.scheme() == "file" {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    if let Some(host) = url.host()
        && let Some(ip) = host_ip_addr(host)
        && private_or_local_ip(ip)
    {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    let Some(host) = url.host_str().map(|host| host.to_ascii_lowercase()) else {
        return PrivacyClass::Ambiguous;
    };
    if matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".corp")
        || (!host.contains('.') && host != "github")
    {
        return PrivacyClass::PrivateOrAuthenticated;
    }
    for (key, _) in url.query_pairs() {
        if secret_query_key(&key) {
            return PrivacyClass::PrivateOrAuthenticated;
        }
    }
    PrivacyClass::Public
}

pub(crate) fn enforce_external_privacy(
    privacy: PrivacyClass,
    explicit_allow_private_external: bool,
    config: &ResearchConfig,
    provider: &str,
) -> Result<()> {
    let allow_private_external =
        explicit_allow_private_external || config.privacy.allow_private_external;
    match privacy {
        PrivacyClass::Public | PrivacyClass::SensitivePublic => Ok(()),
        PrivacyClass::PrivateOrAuthenticated
            if allow_private_external
                || config
                    .privacy
                    .private_external_default
                    .eq_ignore_ascii_case("allow") =>
        {
            Ok(())
        }
        PrivacyClass::Ambiguous
            if allow_private_external
                || config
                    .privacy
                    .ambiguous_external_default
                    .eq_ignore_ascii_case("allow") =>
        {
            Ok(())
        }
        PrivacyClass::PrivateOrAuthenticated => {
            bail!(
                "{provider} refused private/authenticated input; pass --allow-private-external to override"
            )
        }
        PrivacyClass::Ambiguous => {
            bail!(
                "{provider} refused ambiguous input; pass --privacy public or --allow-private-external to override"
            )
        }
    }
}

pub(crate) fn host_ip_addr(host: url::Host<&str>) -> Option<IpAddr> {
    match host {
        url::Host::Ipv4(ip) => Some(IpAddr::V4(ip)),
        url::Host::Ipv6(ip) => Some(IpAddr::V6(ip)),
        url::Host::Domain(_) => None,
    }
}

pub(crate) fn private_or_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
        }
        IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return private_or_local_ip(IpAddr::V4(mapped));
            }
            let first = ip.segments()[0];
            ip.is_loopback()
                || ip.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
        }
    }
}

pub(crate) fn metadata_text(value: &str, config: &ResearchConfig) -> String {
    if config.privacy.redact_query_secrets && text_looks_secret_bearing(value) {
        "[redacted]".to_string()
    } else {
        value.to_string()
    }
}

pub(crate) fn text_looks_secret_bearing(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "token",
        "api_key",
        "apikey",
        "secret",
        "password",
        "authorization",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

pub(crate) fn redact_url_query_secrets(value: &str) -> String {
    let Ok(mut url) = Url::parse(value) else {
        return value.to_string();
    };
    if url.query().is_none() {
        return value.to_string();
    }
    let pairs = url
        .query_pairs()
        .map(|(key, value)| {
            let key = key.to_string();
            let value = if secret_query_key(&key) {
                "[redacted]".to_string()
            } else {
                value.to_string()
            };
            (key, value)
        })
        .collect::<Vec<_>>();
    {
        let mut query = url.query_pairs_mut();
        query.clear();
        for (key, value) in pairs {
            query.append_pair(&key, &value);
        }
    }
    url.to_string()
}

pub(crate) fn redact_metadata_urls(value: Value) -> Value {
    match value {
        Value::String(text) => Value::String(redact_url_query_secrets(&text)),
        Value::Array(values) => {
            Value::Array(values.into_iter().map(redact_metadata_urls).collect())
        }
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, redact_metadata_urls(value)))
                .collect(),
        ),
        other => other,
    }
}

pub(crate) fn secret_query_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "token"
            | "access_token"
            | "auth"
            | "authorization"
            | "signature"
            | "sig"
            | "x-amz-signature"
            | "x-amz-credential"
            | "key"
            | "apikey"
            | "api_key"
            | "secret"
            | "password"
            | "sas"
    )
}

pub(crate) fn privacy_class_name(privacy: PrivacyClass) -> &'static str {
    match privacy {
        PrivacyClass::Public => "public",
        PrivacyClass::SensitivePublic => "sensitive-public",
        PrivacyClass::PrivateOrAuthenticated => "private-or-authenticated",
        PrivacyClass::Ambiguous => "ambiguous",
    }
}

pub(crate) fn provider_name(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::CodexWeb => "codex-web",
        ProviderKind::Context7 => "context7",
        ProviderKind::Github => "github",
        ProviderKind::Exa => "exa",
        ProviderKind::Direct => "direct",
        ProviderKind::Browser => "browser",
        ProviderKind::Firecrawl => "firecrawl",
    }
}
