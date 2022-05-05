use semver::Version;

pub fn valid_version(raw: &str) {
    let split = raw.split(" ").collect::<Vec<_>>();
    assert_eq!(split.len(), 2);
    let version = split[1];

    assert_eq!(split[0], "swc");
    assert!(
        Version::parse(&version).is_ok(),
        "Version should be a valid Semver"
    );
    assert_eq!(version, *swc::VERSION, "Wrong version string");
}
