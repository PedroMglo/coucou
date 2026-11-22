use super::*;
use serde_json::json;
use crate::pet_native_binding::ipc::Revision;

fn native() -> Value {
    let device = json!({"path":"/org/bluez/hci0/dev_11_22_33_44_55_66","address":"11:22:33:44:55:66",
        "name":"Fones","alias":"Auscultadores","paired":true,"trusted":true,"blocked":false,
        "connected":true,"icon":"audio-headset","class":1,"rssi":-40,"legacyPairing":false});
    json!({"powered":true,"discovering":false,
        "adapters":[{"path":"/org/bluez/hci0","name":"Bluetooth","address":"AA:BB:CC:DD:EE:FF","powered":true,"discovering":false}],
        "devices":[device.clone()],"pairedDevices":[device.clone()],"connectedDevices":[device]})
}

fn projected(value: &Value) -> Result<Value, Invalid> {
    project(value, Some(34), &["bluetooth".into()], 1000, Revision(42)).map(|s| serde_json::to_value(s).unwrap())
}

#[test]
fn bluetooth_projection_groups_devices_without_exporting_native_identifiers() {
    let mut input = native();
    input["pairing"] = json!({"token":"PRIVATE_PAIRING_TOKEN","passkey":123456});
    let output = projected(&input).unwrap();
    assert_eq!(output["available"], true);
    assert_eq!(output["navigationRevision"], "42");
    assert_eq!(output["adapters"][0]["deviceCount"], 1);
    assert_eq!(output["devices"][0]["adapterDisplayId"], output["adapters"][0]["displayId"]);
    assert_eq!(output["devices"][0]["label"], "Auscultadores");
    assert_eq!(output["devices"][0]["category"], "headset");
    let text = output.to_string();
    for private in ["/org/bluez", "11:22", "AA:BB", "address", "path", "passkey", "token", "PRIVATE"] {
        assert!(!text.contains(private), "unexpected private projection field");
    }
}

#[test]
fn bluetooth_absent_metadata_and_native_shape_are_unknown_not_empty_success() {
    for version in [None, Some(9), Some(35)] {
        assert_eq!(project(&native(), version, &["bluetooth".into()], 1, Revision(1)).err(), Some(Invalid::Unsupported));
    }
    assert_eq!(project(&native(), Some(34), &["network".into()], 1, Revision(1)).err(), Some(Invalid::Unsupported));
    for field in ["powered", "discovering", "adapters", "devices", "pairedDevices", "connectedDevices"] {
        let mut input = native(); input.as_object_mut().unwrap().remove(field);
        assert_eq!(projected(&input).err(), Some(Invalid::Shape), "{field}");
    }
    let mut input = native(); input["devices"][0]["connected"] = json!("true");
    assert_eq!(projected(&input).err(), Some(Invalid::Shape));
}

#[test]
fn bluetooth_empty_native_lists_are_known_absence_without_a_default_adapter() {
    let input = json!({"powered":false,"discovering":false,"adapters":[],"devices":null,"pairedDevices":null,"connectedDevices":null});
    let output = projected(&input).unwrap();
    assert_eq!(output["available"], false);
    assert_eq!(output["adapters"], json!([]));
    assert_eq!(output["devices"], json!([]));
}

#[test]
fn bluetooth_rejects_over_capacity_duplicates_and_devices_without_their_adapter() {
    let mut input = native(); input["adapters"] = json!(vec![input["adapters"][0].clone(); MAX_ADAPTERS + 1]);
    assert_eq!(projected(&input).err(), Some(Invalid::Capacity));
    let mut input = native(); input["devices"] = json!(vec![input["devices"][0].clone(); MAX_DEVICES + 1]);
    assert_eq!(projected(&input).err(), Some(Invalid::Capacity));
    let mut input = native(); let duplicate = input["devices"][0].clone(); input["devices"].as_array_mut().unwrap().push(duplicate);
    assert_eq!(projected(&input).err(), Some(Invalid::Shape));
    let mut input = native(); input["adapters"] = json!([]);
    assert_eq!(projected(&input).err(), Some(Invalid::Shape));
    let mut input = native(); input["adapters"][0]["path"] = json!("/org/bluez/hci00");
    assert_eq!(projected(&input).err(), Some(Invalid::Shape));
}

#[test]
fn bluetooth_multiple_adapters_keep_individual_power_and_known_device_ownership() {
    let mut input = native();
    input["adapters"].as_array_mut().unwrap().push(json!({"path":"/org/bluez/hci1","name":"Segundo","powered":false,"discovering":false}));
    let output = projected(&input).unwrap();
    assert_eq!(output["adapters"][0]["powered"], true);
    assert_eq!(output["adapters"][1]["powered"], false);
    assert_eq!(output["adapters"][1]["deviceCount"], 0);
    assert!(output.get("powered").is_none());
}

#[test]
fn bluetooth_display_labels_are_optional_and_never_promote_addresses_or_control_text() {
    for label in ["11:22:33:44:55:66", "11-22-33-44-55-66", "\u{202e}spoof", "bad\nlabel", "/org/bluez/hci0"] {
        let mut input = native(); input["devices"][0]["alias"] = json!(label);
        assert_eq!(projected(&input).unwrap()["devices"][0]["label"], "Fones");
    }
    let mut input = native(); input["devices"][0]["alias"] = json!(""); input["devices"][0]["name"] = json!("11:22:33:44:55:66");
    assert_eq!(projected(&input).unwrap()["devices"][0]["label"], Value::Null);
    input["devices"][0]["icon"] = json!("/tmp/private-icon");
    assert_eq!(projected(&input).unwrap()["devices"][0]["category"], "other");
}

#[test]
fn bluetooth_inconsistent_cached_subsets_do_not_invent_a_known_count() {
    let mut input = native(); input["connectedDevices"] = json!([]);
    assert_eq!(projected(&input).err(), Some(Invalid::Shape));
    let mut input = native(); input["pairedDevices"][0]["path"] = json!("/org/bluez/hci0/dev_99_22_33_44_55_66");
    assert_eq!(projected(&input).err(), Some(Invalid::Shape));
}
