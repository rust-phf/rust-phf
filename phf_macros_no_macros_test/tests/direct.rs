#[test]
fn cfgs_work_without_phf_macros_feature() {
    static MAP: phf::Map<&'static str, u8> = phf_macros::phf_map! {
        #[cfg(any())]
        "off" => 0,
        "on" => 1,
    };
    assert_eq!(MAP.get("off"), None);
    assert_eq!(MAP.get("on"), Some(&1));

    static SET: phf::Set<&'static str> = phf_macros::phf_set! {
        #[cfg(any())]
        "off",
        "on",
    };
    assert!(!SET.contains("off"));
    assert!(SET.contains("on"));

    static ORDERED_MAP: phf::OrderedMap<&'static str, u8> = phf_macros::phf_ordered_map! {
        #[cfg(any())]
        "off" => 0,
        "on" => 1,
    };
    assert_eq!(ORDERED_MAP.index(0), Some((&"on", &1)));

    static ORDERED_SET: phf::OrderedSet<&'static str> = phf_macros::phf_ordered_set! {
        #[cfg(any())]
        "off",
        "on",
    };
    assert_eq!(ORDERED_SET.index(0), Some(&"on"));
}

#[test]
fn all_cfgd_out_structures_are_empty() {
    static MAP: phf::Map<&'static str, u8> = phf_macros::phf_map! {
        #[cfg(any())]
        "off" => 0,
    };
    assert_eq!(MAP.len(), 0);

    static SET: phf::Set<&'static str> = phf_macros::phf_set! {
        #[cfg(any())]
        "off",
    };
    assert_eq!(SET.len(), 0);

    static ORDERED_MAP: phf::OrderedMap<&'static str, u8> = phf_macros::phf_ordered_map! {
        #[cfg(any())]
        "off" => 0,
    };
    assert_eq!(ORDERED_MAP.len(), 0);
    assert_eq!(ORDERED_MAP.index(0), None);

    static ORDERED_SET: phf::OrderedSet<&'static str> = phf_macros::phf_ordered_set! {
        #[cfg(any())]
        "off",
    };
    assert_eq!(ORDERED_SET.len(), 0);
    assert_eq!(ORDERED_SET.index(0), None);
}
